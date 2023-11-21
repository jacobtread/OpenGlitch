pub mod raw;
pub mod types;
use raw::{load_memory_struct, FMesh, SafeBuffer};

fn main() {
    // Read entire file into a buffer
    let buffer = std::fs::read("../data/ape/gcdggltch00.ape").unwrap();
    // Drop extra buffer capacity
    let buffer: Box<[u8]> = buffer.into_boxed_slice();

    println!("Buffer length {}", buffer.len());

    let mesh: SafeBuffer<FMesh> = unsafe { load_memory_struct::<FMesh>(buffer) };

    // let value = unsafe { *mesh.skeleton_index_array };

    dbg!(&*mesh);
}
