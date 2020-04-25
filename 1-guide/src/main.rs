use std::sync::Arc;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::ComputePipeline;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

fn get_fit_queue(device: PhysicalDevice) -> (Arc<Device>, Arc<vulkano::device::Queue>) {
    let queue_family = device
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, queues) = {
        Device::new(
            device,
            &Features::none(),
            &DeviceExtensions {
                khr_storage_buffer_storage_class: true,
                ..DeviceExtensions::none()
            },
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed to create device")
    };

    let queues: Vec<_> = queues.collect();
    (device, queues[0].clone())
}

fn main() {
    let instance =
        Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");

    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("failed to create physical device");

    let (device, queue) = get_fit_queue(physical.clone());

    let buffer_numbers =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, 0..1024)
            .expect("failed to create buffer");

    let shader = shader::Shader::load(device.clone()).unwrap();

    let compute_pipeline =
        Arc::new(ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).unwrap());

    use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
    use vulkano::descriptor::PipelineLayoutAbstract;

    let layout = compute_pipeline.layout().descriptor_set_layout(0).unwrap();

    let set = Arc::new(
        PersistentDescriptorSet::start(layout.clone())
            .add_buffer(buffer_numbers.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    use vulkano::command_buffer::AutoCommandBufferBuilder;
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .unwrap()
        .dispatch([1024, 1, 1], compute_pipeline.clone(), set.clone(), ())
        .unwrap()
        .build()
        .unwrap();

    use vulkano::command_buffer::CommandBuffer;
    use vulkano::sync::GpuFuture;

    let finished = command_buffer.execute(queue.clone()).unwrap();

    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let content = buffer_numbers.read().unwrap();

    dbg!(&*content);
}

mod shader {
    use vulkano;
    vulkano_shaders::shader! {
        ty: "compute",
        src: "
#version 450

layout(local_size_x=64, local_size_y=1, local_size_z=1) in;

layout(set=0, binding=0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}
    "
    }
}
