/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate euclid;
extern crate gleam;
extern crate glutin;
extern crate webrender;
extern crate winit;

#[path = "common/boilerplate.rs"]
mod boilerplate;

use boilerplate::Example;
use euclid::TypedScale;
use webrender::api::*;

// This example creates multiple documents overlapping each other with
// specified layer indices.

struct Document {
    id: DocumentId,
    pipeline_id: PipelineId,
    content_rect: LayoutRect,
    color: ColorF,
}

struct App {
    documents: Vec<Document>,
}

impl App {
    fn init(
        &mut self,
        api: &RenderApi,
        device_pixel_ratio: f32,
    ) {
        let init_data = vec![
            (
                PipelineId(1, 0),
                1,
                ColorF::new(0.0, 1.0, 0.0, 1.0),
                FramebufferIntPoint::new(0, 400),
            ),
            (
                PipelineId(2, 0),
                2,
                ColorF::new(1.0, 1.0, 0.0, 1.0),
                FramebufferIntPoint::new(200, 400),
            ),
            (
                PipelineId(3, 0),
                3,
                ColorF::new(1.0, 0.0, 0.0, 1.0),
                FramebufferIntPoint::new(200, 600),
            ),
            (
                PipelineId(4, 0),
                4,
                ColorF::new(1.0, 0.0, 1.0, 1.0),
                FramebufferIntPoint::new(0, 600),
            ),
        ];

        for (pipeline_id, layer, color, offset) in init_data {
            let size = FramebufferIntSize::new(250, 250);
            let bounds = FramebufferIntRect::new(offset, size);

            let document_id = api.add_document(size, layer);
            let mut txn = Transaction::new();
            txn.set_document_view(bounds, device_pixel_ratio);
            txn.set_root_pipeline(pipeline_id);
            api.send_transaction(document_id, txn);

            self.documents.push(Document {
                id: document_id,
                pipeline_id,
                content_rect: LayoutRect::new(
                    LayoutPoint::origin(),
                    bounds.size.to_f32() / TypedScale::new(device_pixel_ratio),
                ),
                color,
            });
        }
    }
}

impl Example for App {
    fn render(
        &mut self,
        api: &RenderApi,
        base_builder: &mut DisplayListBuilder,
        _txn: &mut Transaction,
        framebuffer_size: FramebufferIntSize,
        _pipeline_id: PipelineId,
        _: DocumentId,
    ) {
        if self.documents.is_empty() {
            let device_pixel_ratio = framebuffer_size.width as f32 /
                base_builder.content_size().width;
            // this is the first run, hack around the boilerplate,
            // which assumes an example only needs one document
            self.init(api,  device_pixel_ratio);
        }

        for doc in &self.documents {
            let space_and_clip = SpaceAndClipInfo::root_scroll(doc.pipeline_id);
            let mut builder = DisplayListBuilder::new(
                doc.pipeline_id,
                doc.content_rect.size,
            );
            let local_rect = LayoutRect::new(
                LayoutPoint::zero(),
                doc.content_rect.size,
            );

            builder.push_simple_stacking_context(
                &LayoutPrimitiveInfo::new(doc.content_rect),
                space_and_clip.spatial_id,
            );
            builder.push_rect(
                &LayoutPrimitiveInfo::new(local_rect),
                &space_and_clip,
                doc.color,
            );
            builder.pop_stacking_context();

            let mut txn = Transaction::new();
            txn.set_display_list(
                Epoch(0),
                None,
                doc.content_rect.size,
                builder.finalize(),
                true,
            );
            txn.generate_frame();
            api.send_transaction(doc.id, txn);
        }
    }
}

fn main() {
    let mut app = App {
        documents: Vec::new(),
    };
    boilerplate::main_wrapper(&mut app, None);
}
