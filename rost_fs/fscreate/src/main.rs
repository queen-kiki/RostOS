use rost_fs::disk::block::Block;
use rost_fs::disk::format;
use rost_fs::disk::{Disk, DiskAddress};
use rost_fs::node::*;
use rost_fs::fs::*;

use std::cell::UnsafeCell;
use std::env;
use std::fs::ReadDir;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::ops::Deref;
use std::path::PathBuf;

use walkdir::WalkDir;

pub struct FileDisk {
    block_count: usize,
    file: File,
    data: UnsafeCell<Vec<Block>>,
}

impl FileDisk {
    pub fn new(path: &str, block_count: usize, name: &[u8]) -> io::Result<FileDisk> {
        let fd = FileDisk {
            block_count,
            file: OpenOptions::new().create(true).write(true).open(path)?,
            data: UnsafeCell::new(vec![[0; 4096]; block_count]),
        };

        format(&fd, name).expect("disk formatting failed");

        Ok(fd)
    }

    pub fn save(&mut self) -> io::Result<()> {
        self.file.set_len(0)?;

        let data = unsafe { &*self.data.get() };

        let v: Vec<_> = data
            .iter()
            .flat_map(|block| block.iter())
            .map(|&b| b)
            .collect();

        self.file.write(&v)?;

        Ok(())
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

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 4 {
        println!("Usage: fscreate [image location] [image size in blocks] [root folder path]");
        std::process::exit(-1);
    }

    let image_path = args.get(1).expect("image path missing");

    let size = args
        .get(2)
        .expect("image size missing")
        .parse()
        .expect("image size is not a number");

    let root_path = args.get(3).expect("root path missing");

    let mut file_disk = FileDisk::new(image_path, size, b"TEST DISK").expect("disk creation failed");

    let mut tree = NodeTree::new(&mut file_disk);

    tree.insert_node(0).expect("inserting root dir failed");
    
    write(0, NodeHeader::DIRECTORY, &[], &mut tree);

    //println!("added /");;

    for entry in WalkDir::new(root_path)
        .into_iter()
        .skip(1)
        .filter_map(|e| e.ok())
    {
        let node_type = entry.file_type();
        let local_path = entry.path().to_owned();
        let path: PathBuf = local_path.iter().skip(1).collect();

        assert!(path.to_str().unwrap().is_ascii());

        let new_entry = create(
            &mut tree,
            &path.to_str().unwrap().bytes().map(|c| match c {
                b'\\'=>b'/',
                _ => c,
            }).collect::<Vec<_>>(),
            0,
        )
        .expect("failed to write file");

        if node_type.is_file() {
            let mut file = File::open(entry.path()).expect("failed to open file");
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).expect("failed to read file");
            //println!("wrote to {}", entry.path().display());
            write(new_entry, NodeHeader::FILE, &buf, &mut tree);
        } else if node_type.is_dir() {
            //println!("created {}", entry.path().display());
            write(new_entry, NodeHeader::DIRECTORY, &[], &mut tree);
        } else {
            panic!("invalid entry")
        }
    }

    let blocks_used = rost_fs::disk::block::get_root_block(&file_disk).top_block.index().expect("no root block found (impossible error)");

    let fraction = blocks_used as f64 / file_disk.block_count as f64;
    let total_size = file_disk.block_count as f64 * 4096.;
    println!("Used ~{}% of image. ({}/{} MiB)", fraction * 100., total_size * fraction / 1000000., total_size / 1000000.);

    file_disk.save().expect("saving failed");
}
