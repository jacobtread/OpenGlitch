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
    let buffer = std::fs::read("data/ape/grdggltch00.ape").unwrap();
    // Drop extra buffer capacity
    let buffer: Box<[u8]> = buffer.into_boxed_slice();

    println!("Buffer length {}", buffer.len());

    let mesh: SafeBuffer<FMesh> = unsafe { load_memory_struct::<FMesh>(buffer) };

    // let value = unsafe { *mesh.skeleton_index_array };

    let mesh: &FMesh = &mesh;

    writeln!(&mut debug_dump, "{:#?}", &mesh).unwrap();

    // dbg!(&mesh.materials());

    let dx_mesh: &mut raw::dx::DxMesh = mesh.impl_specific_mut().unwrap();
    // let buffers: &[raw::dx::DxVertexBufferDescriptor] = dx_mesh.vertex_buffers().unwrap();
    writeln!(&mut debug_dump, "{:#?}", dx_mesh).unwrap();
    // writeln!(&mut debug_dump, "{:#?}", buffers).unwrap();

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

    dbg!(&dx_mesh);

    let index_buffers = dx_mesh.index_buffers();

    let index_buffer = *index_buffers.first().unwrap();
    for value in index_buffer {
        writeln!(&mut buffer_dump_index, "{}", value);
    }

    let vertex_buffer = dx_mesh.vertex_buffers_mut().unwrap();
    writeln!(&mut debug_dump, "{:#?}", vertex_buffer).unwrap();

    // println!("{}", values.len());

    // for value in indices {
    //     if value as usize > values.len() {
    //         // panic!("Index out of range {}", value);
    //     }
    //     writeln!(&mut buffer_dump_index, "{}", value);
    // }

    for buffer in vertex_buffer {
        writeln!(&mut buffer_dump, "Buffer 1").unwrap();
        let positions = buffer.positions();

        for [a, b, c] in positions {
            writeln!(&mut buffer_dump, "{} {} {}", a, b, c).unwrap();
        }
    }
}
