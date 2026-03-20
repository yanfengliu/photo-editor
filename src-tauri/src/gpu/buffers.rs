use wgpu::*;

pub struct TexturePair {
    pub input: Texture,
    pub output: Texture,
    pub input_view: TextureView,
    pub output_view: TextureView,
    pub width: u32,
    pub height: u32,
}

impl TexturePair {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let desc = TextureDescriptor {
            label: Some("Processing Texture"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC | TextureUsages::COPY_DST,
            view_formats: &[],
        };
        let input = device.create_texture(&desc);
        let output = device.create_texture(&desc);
        let input_view = input.create_view(&TextureViewDescriptor::default());
        let output_view = output.create_view(&TextureViewDescriptor::default());
        Self { input, output, input_view, output_view, width, height }
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.input, &mut self.output);
        std::mem::swap(&mut self.input_view, &mut self.output_view);
    }
}
