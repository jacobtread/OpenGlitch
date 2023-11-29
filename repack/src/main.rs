pub mod raw;
pub mod st;
pub mod types;
use std::fs::{File, OpenOptions};

use st::{load_memory_struct, FMesh, SafeBuffer};

fn main() {
    use std::io::Write;

    let mut debug_dump = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("dump.txt")
        .unwrap();

    // Read entire file into a buffer
    let buffer = std::fs::read("../data/ape/gcdggltch00.ape").unwrap();
    // Drop extra buffer capacity
    let buffer: Box<[u8]> = buffer.into_boxed_slice();

    println!("Buffer length {}", buffer.len());

    let mesh: SafeBuffer<FMesh> = unsafe { load_memory_struct::<FMesh>(buffer) };

    // let value = unsafe { *mesh.skeleton_index_array };

    let mesh: &FMesh = &mesh;

    writeln!(&mut debug_dump, "{:#?}", &mesh).unwrap();

    let gx_mesh: &raw::gc::GxMesh = mesh.impl_specific().unwrap();
    let buffers: &[raw::gc::GxVertexBuffer] = gx_mesh.vertex_buffers().unwrap();
    writeln!(&mut debug_dump, "{:#?}", gx_mesh).unwrap();
    writeln!(&mut debug_dump, "{:#?}", buffers).unwrap();

    let mut buffer_dump = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("buffer_dump.txt")
        .unwrap();

    let mut buffer_dump_index = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("buffer_dump_index.txt")
        .unwrap();

    let first_buffer = &buffers[0];

    let values = first_buffer.normalized();
    let indices = first_buffer.normalized_indicies();

    for value in &values {
        writeln!(&mut buffer_dump, "{},{},{}", value.x, value.y, value.z);
    }
    println!("{}", values.len());

    for value in indices {
        if value as usize > values.len() {
            // panic!("Index out of range {}", value);
        }
        writeln!(&mut buffer_dump_index, "{}", value);
    }

    for buffer in buffers {
        let positions = buffer.positions().unwrap();

        writeln!(&mut debug_dump, "{:?}", positions).unwrap();
    }
}
