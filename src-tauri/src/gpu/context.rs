use crate::gpu::passes::basic_adjustments::BasicAdjustmentsPass;

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_name: String,
    pub backend: String,
    pub basic_adjustments_pass: BasicAdjustmentsPass,
}

impl GpuContext {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }).await.ok_or("No GPU adapter found")?;
        let info = adapter.get_info();
        let adapter_name = info.name.clone();
        let backend = format!("{:?}", info.backend);
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Photo Editor GPU"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
        }, None).await?;
        let basic_adjustments_pass = BasicAdjustmentsPass::new(&device);
        log::info!("GPU initialized: {} ({})", adapter_name, backend);
        Ok(Self { device, queue, adapter_name, backend, basic_adjustments_pass })
    }
}
