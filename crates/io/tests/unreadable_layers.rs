use photocraft_color::{ColorMode, PixelFormat, SampleType};
use photocraft_doc::{Document, Layer, LayerContent};
use photocraft_geom::Size;
use photocraft_psd::TaggedBlock;

#[test]
fn unreadable_lr16_block_emits_warning_and_prevents_duplicate_on_save() {
    let mut doc = Document::new("Test", Size::new(8, 8), ColorMode::Rgb, SampleType::U16);
    let fmt = PixelFormat::new(ColorMode::Rgb, SampleType::U16, true);
    doc.layers.push(Layer::new("Test Layer", LayerContent::Raster(photocraft_raster::Surface::new(fmt))));

    // Export normal PSD
    let psd = photocraft_io::export(&doc, "test.psb", &Default::default()).unwrap();
    let mut file = photocraft_psd::PsdFile::from_bytes(&psd.bytes).unwrap();

    // Move layer info to global block and corrupt it
    let _info = file.layer_info.take().unwrap();
    let mut corrupted = Vec::new();
    // length of layers (count 5), but not enough bytes!
    corrupted.extend_from_slice(&5i16.to_be_bytes());
    corrupted.extend_from_slice(&[0; 2]); // Not enough bytes for 5 records

    file.global_blocks.push(TaggedBlock::new(*b"Lr16", corrupted));

    let corrupted_bytes = file.to_bytes().unwrap();

    // Import again
    let result = photocraft_io::import("test.psb", &corrupted_bytes).unwrap();
    let imported = result.document;
    let warnings = result.warnings;

    assert!(warnings.iter().any(|w| w.contains("the layers could not be read")), "Missing warning: {:?}", warnings);

    // Export again without adding layers
    let psd2 = photocraft_io::export(&imported, "test2.psb", &Default::default()).unwrap();
    let file2 = photocraft_psd::PsdFile::from_bytes(&psd2.bytes).unwrap();

    let lr16_count = file2.global_blocks.iter().filter(|b| b.key == *b"Lr16").count();
    // It should not produce a duplicate Lr16 block.
    // (It either keeps the old one if saving flattened, or writes a new one for the Background layer, but never two).
    assert!(lr16_count <= 1, "Expected at most one Lr16 block (no duplicates)");

    // Now add a layer and export
    let mut imported_doc = imported;
    imported_doc.layers.push(Layer::new("New Layer", LayerContent::Raster(photocraft_raster::Surface::new(fmt))));
    let psd3 = photocraft_io::export(&imported_doc, "test3.psb", &Default::default()).unwrap();
    let file3 = photocraft_psd::PsdFile::from_bytes(&psd3.bytes).unwrap();

    let lr16_count3 = file3.global_blocks.iter().filter(|b| b.key == *b"Lr16").count();
    assert!(lr16_count3 <= 1, "Expected at most one Lr16 block (no duplicates)");
}
