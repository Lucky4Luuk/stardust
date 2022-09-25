pub struct RenderPass {
    name: String,
    view: wgpu::TextureView
}

impl RenderPass {
    pub fn from_view<S: Into<String>>(name: S, view: wgpu::TextureView) -> Self {
        Self {
            name: name.into(),
            view: view,
        }
    }

    // TODO: Perhaps formatting a string in a render function is not the best idea lol
    pub fn render(self, renderer: &super::Renderer) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("{} Encoder", self.name)),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("{} Render Pass", self.name)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.2,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        renderer.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
