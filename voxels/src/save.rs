use std::fs::File;
use std::io::{Read, Write};

use crate::voxel::*;
use crate::world_gen::WORLD_DIMS;

pub fn save(chunks: &Vec<Chunk>) {
    let mut file = std::fs::File::create("save.txt").expect("creation failed");
    for chunk in chunks {
        for x in 0..CHUNK_SIZE as usize {
            for y in 0..CHUNK_SIZE as usize {
                file.write_all(&chunk.data[x][y]).expect("write failed");
            }
        }
    }
    println!("sucessful save");
}

pub fn load() -> Vec<Chunk> {
    let mut f = File::open("save.txt").expect("no file found");
    let metadata = std::fs::metadata("save.txt").expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    let mut chunks: Vec<Chunk> = Vec::with_capacity(WORLD_DIMS.0 * WORLD_DIMS.1 * WORLD_DIMS.2);
    let mut data: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] =
        [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
    for i in 0..buffer.len() {
        let x = (i / (CHUNK_SIZE.pow(2))) % CHUNK_SIZE;
        let y = (i / CHUNK_SIZE) % CHUNK_SIZE;
        let z = i % CHUNK_SIZE;
        data[x][y][z] = buffer[i];
        if (i + 1) % CHUNK_SIZE.pow(3) == 0 {
            chunks.push(Chunk { data , voxels:vec![]});
        }
    }
    println!("succesful load");
    chunks
}
