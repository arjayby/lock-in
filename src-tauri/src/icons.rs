//! App icons as small PNGs, taken from the `.app` bundle through NSWorkspace.

use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
use objc2_foundation::{NSDictionary, NSString};

/// Pixel width to aim for: 32 pt icons on a 2x display.
const TARGET_PX: isize = 64;

pub fn app_icon_png(bundle_path: &str) -> Option<Vec<u8>> {
    let image = NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(bundle_path));
    let tiff = image.TIFFRepresentation()?;
    let reps = NSBitmapImageRep::imageRepsWithData(&tiff);

    // The smallest representation that is at least the target size, else the largest one.
    let mut reps: Vec<_> = reps.iter().filter_map(|r| r.downcast::<NSBitmapImageRep>().ok()).collect();
    reps.sort_by_key(|r| r.pixelsWide());
    let rep = reps.iter().find(|r| r.pixelsWide() >= TARGET_PX).or(reps.last())?;

    let png = unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new()) }?;
    Some(png.to_vec())
}

#[cfg(test)]
mod tests {
    #[test]
    fn extracts_a_system_app_icon() {
        let png = super::app_icon_png("/System/Applications/Calculator.app").expect("an icon");
        assert_eq!(&png[1..4], b"PNG");
    }
}
