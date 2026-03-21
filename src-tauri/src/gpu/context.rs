use crate::gpu::buffers::TexturePair;
use crate::gpu::passes::basic_adjustments::BasicAdjustmentsPass;

/// Cached GPU resources for a specific image dimension, reused across frames.
pub struct CachedResources {
    pub textures: TexturePair,
    pub readback: wgpu::Buffer,
    pub padded_bytes_per_row: u32,
    pub width: u32,
    pub height: u32,
}

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_name: String,
    pub backend: String,
    pub basic_adjustments_pass: BasicAdjustmentsPass,
    pub cached: Option<CachedResources>,
}

fn align_to(value: u32, alignment: u32) -> u32 {
    if value.is_multiple_of(alignment) { value } else { value + alignment - (value % alignment) }
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
        Ok(Self { device, queue, adapter_name, backend, basic_adjustments_pass, cached: None })
    }

    /// Get or create cached GPU resources for the given dimensions.
    pub fn get_or_create_resources(&mut self, width: u32, height: u32) -> &CachedResources {
        let needs_recreate = match &self.cached {
            Some(c) => c.width != width || c.height != height,
            None => true,
        };

        if needs_recreate {
            let textures = TexturePair::new(&self.device, width, height, wgpu::TextureFormat::Rgba8Unorm);
            let bytes_per_row = width * 4;
            let padded_bytes_per_row = align_to(bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
            let readback_size = padded_bytes_per_row as u64 * height as u64;
            let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Develop Readback Buffer"),
                size: readback_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            self.cached = Some(CachedResources {
                textures,
                readback,
                padded_bytes_per_row,
                width,
                height,
            });
        }

        self.cached.as_ref().unwrap()
    }
}
