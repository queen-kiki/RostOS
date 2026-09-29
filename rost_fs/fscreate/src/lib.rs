use rost_fs::disk::block::Block;
use rost_fs::disk::format;
use rost_fs::disk::{Disk, DiskAddress};
use rost_fs::fs::*;
use rost_fs::node::*;

use std::cell::UnsafeCell;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use walkdir::WalkDir;

struct FileDisk {
    block_count: usize,
    file: File,
    data: UnsafeCell<Vec<Block>>,
}

impl FileDisk {
    fn new(path: &Path, block_count: usize, name: &[u8]) -> io::Result<FileDisk> {
        let fd = FileDisk {
            block_count,
            file: OpenOptions::new().create(true).write(true).truncate(true).open(path)?,
            data: UnsafeCell::new(vec![[0; 4096]; block_count]),
        };

        format(&fd, name).expect("disk formatting failed");

        Ok(fd)
    }

    fn save(&mut self) -> io::Result<()> {
        self.file.write_all(self.data.get_mut().as_flattened())
    }
}

impl Disk for FileDisk {
    fn get_block(&self, index: DiskAddress) -> Option<&mut Block> {
        unsafe { (&mut *self.data.get()).get_mut(index.index()? as usize) }
    }

    fn block_count(&self) -> u64 {
        self.block_count as _
    }
}

/// Writes a RostFS image of `block_count` blocks to `image_path`, containing
/// everything below `root_path`. Returns the fraction of the image in use.
pub fn create_image(image_path: &Path, block_count: usize, root_path: &Path) -> io::Result<f64> {
    let mut file_disk = FileDisk::new(image_path, block_count, b"TEST DISK")?;

    let mut tree = NodeTree::new(&mut file_disk);

    tree.insert_node(0).expect("inserting root dir failed");

    write(0, NodeHeader::DIRECTORY, &[], &mut tree);

    // WalkDir yields parents before their children, so every directory exists
    // by the time something is created inside it.
    for entry in WalkDir::new(root_path).min_depth(1) {
        let entry = entry?;
        let node_type = entry.file_type();
        let path = entry.path().strip_prefix(root_path).unwrap();

        let path = path.to_str().filter(|p| p.is_ascii()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("non-ascii path: {}", path.display()),
            )
        })?;

        let new_entry = create(
            &mut tree,
            &path
                .bytes()
                .map(|c| match c {
                    b'\\' => b'/',
                    _ => c,
                })
                .collect::<Vec<_>>(),
            0,
        )
        .expect("failed to write file");

        if node_type.is_file() {
            write(new_entry, NodeHeader::FILE, &fs::read(entry.path())?, &mut tree);
        } else if node_type.is_dir() {
            write(new_entry, NodeHeader::DIRECTORY, &[], &mut tree);
        } else {
            panic!("invalid entry")
        }
    }

    let blocks_used = rost_fs::disk::block::get_root_block(&file_disk)
        .top_block
        .index()
        .expect("no root block found (impossible error)");

    file_disk.save()?;

    Ok(blocks_used as f64 / file_disk.block_count as f64)
}
