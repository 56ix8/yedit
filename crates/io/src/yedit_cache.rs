//! Per-document native export cache; file writes and save identity stay with the caller.

use std::sync::{Arc, Mutex};

use yedit_doc::Document;
use yedit_format::{YeditWriter, SaveOptions};

/// Shared native export cache. Clones reuse tile hashes and up to 64 MiB of compressed payloads;
/// dropping the last clone releases it, including when a document closes after a save worker.
#[derive(Clone)]
pub struct YeditExportCache(Arc<Mutex<YeditWriter>>);

impl Default for YeditExportCache {
    fn default() -> Self {
        let mut writer = YeditWriter::new();
        writer.set_compressed_cache_budget(64 << 20);
        Self(Arc::new(Mutex::new(writer)))
    }
}

impl std::fmt::Debug for YeditExportCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("YeditExportCache")
    }
}

impl PartialEq for YeditExportCache {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl YeditExportCache {
    pub(crate) fn encode(&self, doc: &Document, opts: &SaveOptions) -> yedit_format::Result<Vec<u8>> {
        self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).save_zip(doc, opts).map(|(bytes, _)| bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExportOptions, export};
    use yedit_color::{ColorMode, SampleType};
    use yedit_doc::{Layer, LayerContent};
    use yedit_geom::Size;
    use yedit_raster::Surface;

    #[test]
    fn cached_exports_reuse_tiles_and_preserve_edits_and_save_as() {
        for depth in SampleType::ALL {
            let mut doc = Document::new("first.yedit", Size::new(2, 1), ColorMode::Rgb, depth);
            let mut surface = Surface::new(doc.pixel_format());
            surface.write_pixel(0, 0, &[0.2, 0.4, 0.6, 1.0]);
            doc.layers.push(Layer::new("photo", LayerContent::Raster(surface)));
            let cache = YeditExportCache::default();
            let options = ExportOptions { yedit_cache: Some(cache.clone()), ..Default::default() };
            let first = export(&doc, "first.yedit", &options).unwrap();
            assert_eq!(yedit_format::load_from_bytes(&first.bytes).unwrap(), doc);
            let (_, stats) = cache.0.lock().unwrap().save_zip(&doc, &SaveOptions::default()).unwrap();
            assert_eq!(stats.tiles_reused, stats.tiles_total);
            assert_eq!(stats.tiles_written, 0);
            doc.layers[0].surface_mut().unwrap().try_write_region(yedit_geom::Rect::new(0, 0, 1, 1), &[0.8, 0.3, 0.1, 1.0]).unwrap();
            let next = export(&doc, "second.yedit", &options).unwrap();
            let mut expected = doc.clone();
            expected.name = "second.yedit".into();
            assert_eq!(yedit_format::load_from_bytes(&next.bytes).unwrap(), expected);
            assert_eq!(next.bytes, export(&doc, "second.yedit", &ExportOptions::default()).unwrap().bytes);
            let weak = Arc::downgrade(&cache.0);
            drop(options);
            drop(cache);
            assert!(weak.upgrade().is_none());
        }
    }
}
