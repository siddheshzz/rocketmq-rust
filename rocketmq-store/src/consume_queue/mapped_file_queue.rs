/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use log::warn;
use parking_lot::RwLock;
use rocketmq_common::UtilAll::offset_to_file_name;
use tracing::info;

use crate::{
    log_file::mapped_file::{default_impl::DefaultMappedFile, MappedFile},
    services::allocate_mapped_file_service::AllocateMappedFileService,
};

#[derive(Default, Clone)]
pub struct MappedFileQueue {
    pub(crate) store_path: String,

    pub(crate) mapped_file_size: u64,
    //pub(crate) mapped_files: Arc<Mutex<Vec<LocalMappedFile>>>,
    //pub(crate) mapped_files: Vec<Arc<Mutex<LocalMappedFile>>>,
    //pub(crate) mapped_files: Vec<Arc<LocalMappedFile>>,
    //pub(crate) mapped_files: Vec<Arc<DefaultMappedFile>>,
    pub(crate) mapped_files: Arc<RwLock<Vec<Arc<DefaultMappedFile>>>>,
    //  pub(crate) mapped_files: Vec<LocalMappedFile>,
    pub(crate) allocate_mapped_file_service: Option<AllocateMappedFileService>,

    pub(crate) flushed_where: Arc<AtomicU64>,

    pub(crate) committed_where: Arc<AtomicU64>,

    pub(crate) store_timestamp: Arc<AtomicU64>,
}

impl MappedFileQueue {
    pub fn new(
        store_path: String,
        mapped_file_size: u64,
        allocate_mapped_file_service: Option<AllocateMappedFileService>,
    ) -> MappedFileQueue {
        MappedFileQueue {
            store_path,
            mapped_file_size,
            mapped_files: Arc::new(RwLock::new(Vec::new())),
            allocate_mapped_file_service,
            flushed_where: Arc::new(AtomicU64::new(0)),
            committed_where: Arc::new(AtomicU64::new(0)),
            store_timestamp: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn load(&mut self) -> bool {
        //list dir files
        let dir = Path::new(&self.store_path);
        if let Ok(ls) = fs::read_dir(dir) {
            let files: Vec<_> = ls
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect();
            return self.do_load(files);
        }
        true
    }

    pub fn check_self(&self) {
        println!("mapped_file_queue check self unimplemented")
    }

    pub fn do_load(&mut self, files: Vec<std::path::PathBuf>) -> bool {
        // Ascending order sorting
        let mut files = files;
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        let mut index = 0;
        for file in &files {
            index += 1;
            if file.is_dir() {
                continue;
            }

            if file.metadata().map(|metadata| metadata.len()).unwrap_or(0) == 0
                && files.len() == index
            {
                if let Err(err) = fs::remove_file(file) {
                    warn!(
                        "{} size is 0, auto delete. is_ok: false, err: {}",
                        file.display(),
                        err
                    );
                } else {
                    warn!("{} size is 0, auto delete. is_ok: true", file.display());
                }
                continue;
            }

            let file_size = file.metadata().map(|metadata| metadata.len()).unwrap_or(0);
            if file_size != self.mapped_file_size {
                warn!(
                    "{} {} length not matched message store config value, please check it manually",
                    file.display(),
                    file_size
                );
                return false;
            }

            let mapped_file =
                DefaultMappedFile::new(file.to_string_lossy().to_string(), self.mapped_file_size);
            // Set wrote, flushed, committed positions for mapped_file
            mapped_file.set_wrote_position(self.mapped_file_size as i32);
            mapped_file.set_flushed_position(self.mapped_file_size as i32);
            mapped_file.set_committed_position(self.mapped_file_size as i32);
            self.mapped_files.write().push(Arc::new(mapped_file));
            // self.mapped_files
            //     .push(mapped_file);
            info!("load {} OK", file.display());
        }

        true
    }

    pub fn get_last_mapped_file(&self) -> Option<Arc<DefaultMappedFile>> {
        if self.mapped_files.read().is_empty() {
            return None;
        }
        self.mapped_files.read().last().cloned()
    }

    pub fn get_first_mapped_file(&self) -> Option<Arc<DefaultMappedFile>> {
        if self.mapped_files.read().is_empty() {
            return None;
        }
        self.mapped_files.read().first().cloned()
    }

    pub fn get_last_mapped_file_mut_start_offset(
        &mut self,
        start_offset: u64,
        need_create: bool,
    ) -> Option<Arc<DefaultMappedFile>> {
        let mut create_offset = -1i64;
        let file_size = self.mapped_file_size as i64;
        let mapped_file_last = self.get_last_mapped_file();
        match mapped_file_last {
            None => {
                create_offset = start_offset as i64 - (start_offset as i64 % file_size);
            }
            Some(ref value) => {
                if value.is_full() {
                    create_offset = value.get_file_from_offset() as i64 + file_size
                }
            }
        }
        if create_offset != -1 && need_create {
            return self.try_create_mapped_file(create_offset as u64);
        }
        mapped_file_last
    }

    pub fn try_create_mapped_file(&mut self, create_offset: u64) -> Option<Arc<DefaultMappedFile>> {
        let next_file_path =
            PathBuf::from(self.store_path.clone()).join(offset_to_file_name(create_offset));
        let next_next_file_path = PathBuf::from(self.store_path.clone())
            .join(offset_to_file_name(create_offset + self.mapped_file_size));
        self.do_create_mapped_file(next_file_path, next_next_file_path)
    }

    fn do_create_mapped_file(
        &mut self,
        next_file_path: PathBuf,
        _next_next_file_path: PathBuf,
    ) -> Option<Arc<DefaultMappedFile>> {
        let mut mapped_file = match self.allocate_mapped_file_service {
            None => DefaultMappedFile::new(
                next_file_path.to_string_lossy().to_string(),
                self.mapped_file_size,
            ),
            Some(ref _value) => {
                unimplemented!()
            }
        };

        if self.mapped_files.read().is_empty() {
            mapped_file.set_first_create_in_queue(true);
        }
        let inner = Arc::new(mapped_file);
        self.mapped_files.write().push(inner.clone());
        Some(inner)
    }

    pub fn get_mapped_files(&self) -> Arc<RwLock<Vec<Arc<DefaultMappedFile>>>> {
        self.mapped_files.clone()
    }

    pub fn get_mapped_files_size(&self) -> usize {
        self.mapped_files.read().len()
    }

    pub fn set_flushed_where(&self, flushed_where: i64) {
        self.flushed_where
            .store(flushed_where as u64, Ordering::SeqCst);
    }

    pub fn set_committed_where(&self, committed_where: i64) {
        self.committed_where
            .store(committed_where as u64, Ordering::SeqCst);
    }

    pub fn truncate_dirty_files(&mut self, offset: i64) {
        let mut will_remove_files = Vec::new();
        for mapped_file in self.mapped_files.read().iter() {
            let file_tail_offset = mapped_file.get_file_from_offset() + self.mapped_file_size;
            if file_tail_offset as i64 > offset {
                if offset >= mapped_file.get_file_from_offset() as i64 {
                    mapped_file.set_wrote_position((offset % self.mapped_file_size as i64) as i32);
                    mapped_file
                        .set_committed_position((offset % self.mapped_file_size as i64) as i32);
                    mapped_file
                        .set_flushed_position((offset % self.mapped_file_size as i64) as i32);
                } else {
                    mapped_file.destroy(1000);
                    will_remove_files.push(mapped_file.clone());
                }
            }
        }
    }

    pub fn get_max_offset(&self) -> i64 {
        match self.get_last_mapped_file() {
            None => 0,
            Some(file) => file.get_file_from_offset() as i64 + file.get_read_position() as i64,
        }
    }

    pub fn delete_last_mapped_file(&mut self) {
        if let Some(last_mapped_file) = self.get_last_mapped_file() {
            last_mapped_file.destroy(1000);
            self.mapped_files
                .write()
                .retain(|mf| mf.as_ref() != last_mapped_file.as_ref());
            info!(
                "on recover, destroy a logic mapped file {}",
                last_mapped_file.get_file_name()
            );
        }
    }

    pub(crate) fn delete_expired_file(&mut self, files: Vec<Arc<DefaultMappedFile>>) {
        let mut files = files;
        let read_guard = self.mapped_files.read();
        if !files.is_empty() {
            files.retain(|mf| read_guard.contains(mf));
            self.mapped_files.write().retain(|mf| !files.contains(mf));
        }
    }

    pub fn destroy(&mut self) {
        for mapped_file in self.mapped_files.read().iter() {
            mapped_file.destroy(1000 * 3);
        }
        self.mapped_files.write().clear();
        self.set_flushed_where(0);
        let path = PathBuf::from(&self.store_path);
        if path.is_dir() {
            let _ = fs::remove_dir_all(path);
        }
    }

    pub fn find_mapped_file_by_offset(
        &self,
        offset: i64,
        return_first_on_not_found: bool,
    ) -> Option<Arc<DefaultMappedFile>> {
        let first_mapped_file = self.get_first_mapped_file();
        let last_mapped_file = self.get_last_mapped_file();
        if first_mapped_file.is_some() && last_mapped_file.is_some() {
            if offset < first_mapped_file.as_ref().unwrap().get_file_from_offset() as i64
                || offset
                    >= last_mapped_file.as_ref().unwrap().get_file_from_offset() as i64
                        + self.mapped_file_size as i64
            {
                if return_first_on_not_found {
                    first_mapped_file
                } else {
                    None
                }
            } else {
                let index = offset as usize / self.mapped_file_size as usize
                    - first_mapped_file.as_ref().unwrap().get_file_from_offset() as usize
                        / self.mapped_file_size as usize;
                let read_guard = self.mapped_files.read();
                let target_file = read_guard.get(index).cloned();
                if target_file.is_some()
                    && offset >= target_file.as_ref().unwrap().get_file_from_offset() as i64
                {
                    return target_file;
                }
                for index in 0..read_guard.len() {
                    let mapped_file = read_guard.get(index).unwrap();
                    if offset >= mapped_file.get_file_from_offset() as i64
                        && offset
                            < mapped_file.get_file_from_offset() as i64
                                + self.mapped_file_size as i64
                    {
                        return Some(mapped_file.clone());
                    }
                }
                if return_first_on_not_found {
                    return first_mapped_file;
                }
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::*;

    #[test]
    fn test_load_empty_dir() {
        let mut queue = MappedFileQueue {
            store_path: String::from("/path/to/empty/dir"),
            ..MappedFileQueue::default()
        };
        assert!(queue.load());
        assert!(queue.mapped_files.read().is_empty());
    }

    #[test]
    fn test_load_with_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file1_path = temp_dir.path().join("1111");
        let file2_path = temp_dir.path().join("2222");
        fs::File::create(&file1_path).unwrap();
        fs::File::create(&file2_path).unwrap();

        let mut queue = MappedFileQueue {
            store_path: temp_dir.path().to_string_lossy().into_owned(),
            ..MappedFileQueue::default()
        };
        assert!(queue.load());
        assert_eq!(queue.mapped_files.read().len(), 1);
    }

    #[test]
    fn test_load_with_empty_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("1111");
        fs::File::create(&file_path).unwrap();

        let mut queue = MappedFileQueue {
            store_path: temp_dir.path().to_string_lossy().into_owned(),
            ..MappedFileQueue::default()
        };
        assert!(queue.load());
        assert!(queue.mapped_files.read().is_empty());
    }

    #[test]
    fn test_load_with_invalid_file_size() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("invalid_file.txt");
        fs::write(&file_path, "Some data").unwrap();

        let mut queue = MappedFileQueue {
            store_path: temp_dir.path().to_string_lossy().into_owned(),
            ..MappedFileQueue::default()
        };
        assert!(!queue.load());
        assert!(queue.mapped_files.read().is_empty());
    }

    #[test]
    fn test_load_with_correct_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("1111");
        fs::write(&file_path, vec![0u8; 1024]).unwrap();

        let mut queue = MappedFileQueue {
            store_path: temp_dir.path().to_string_lossy().into_owned(),
            mapped_file_size: 1024,
            ..MappedFileQueue::default()
        };
        assert!(queue.load());
        assert_eq!(queue.mapped_files.read().len(), 1);
    }
}
