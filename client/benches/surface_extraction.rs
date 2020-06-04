use std::sync::Arc;

use ash::{version::DeviceV1_0, vk};
use bencher::{benchmark_group, benchmark_main, Bencher};

use client::graphics::{
    voxels::surface_extraction::{self, SurfaceExtraction},
    Base,
};
//use common::world::Material;

fn extract(bench: &mut Bencher) {
    let gfx = Arc::new(Base::headless());
    let extract = SurfaceExtraction::new(&gfx);
    let mut scratch = surface_extraction::ScratchBuffer::new(&gfx, &extract, BATCH_SIZE, DIMENSION);
    let draw = surface_extraction::DrawBuffer::new(&gfx, BATCH_SIZE, DIMENSION);
    let device = &*gfx.device;

    unsafe {
        let cmd_pool = device
            .create_command_pool(
                &vk::CommandPoolCreateInfo::builder().queue_family_index(gfx.queue_family),
                None,
            )
            .unwrap();

        let cmd = device
            .allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::builder()
                    .command_pool(cmd_pool)
                    .command_buffer_count(1),
            )
            .unwrap()[0];

        device
            .begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default())
            .unwrap();

        for i in 0..BATCH_SIZE {
            scratch.extract(
                device,
                &extract,
                i,
                false,
                DIMENSION,
                cmd,
                (draw.indirect_buffer(), draw.indirect_offset(i)),
                (draw.face_buffer(), draw.face_offset(i)),
            );
        }
        device.end_command_buffer(cmd).unwrap();

        bench.iter(|| {
            device
                .queue_submit(
                    gfx.queue,
                    &[vk::SubmitInfo::builder().command_buffers(&[cmd]).build()],
                    vk::Fence::null(),
                )
                .unwrap();
            device.device_wait_idle().unwrap();
        })
    }
}

const DIMENSION: u32 = 16;
const BATCH_SIZE: u32 = 16;

benchmark_group!(benches, extract);
benchmark_main!(benches);