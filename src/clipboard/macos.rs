use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result, bail};
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{NSPasteboard, NSPasteboardItem};
use objc2_foundation::{NSArray, NSData, NSString};

use crate::filebundle;
use crate::model::Representation;

use super::{is_file_format, is_internal_marker, is_sensitive_marker};

pub(super) fn publish_single_file(path: &Path, representations: &[Representation]) -> Result<()> {
    let pasteboard = NSPasteboard::generalPasteboard();
    publish_single_file_to(&pasteboard, path, representations)
}

fn publish_single_file_to(
    pasteboard: &NSPasteboard,
    path: &Path,
    representations: &[Representation],
) -> Result<()> {
    if !path.is_file() {
        bail!("clipboard file does not exist: {}", path.display());
    }

    let item = NSPasteboardItem::new();
    let file_url_type = NSString::from_str("public.file-url");
    let file_url = NSString::from_str(&filebundle::path_to_uri(path));
    if !item.setString_forType(&file_url, &file_url_type) {
        bail!("publish file URL to macOS pasteboard");
    }

    let mut image_types = HashSet::new();
    for representation in representations {
        if is_file_format(&representation.format)
            || is_internal_marker(&representation.format)
            || is_sensitive_marker(&representation.format)
        {
            continue;
        }
        let Some(format) = native_image_format(&representation.format) else {
            continue;
        };
        if !image_types.insert(format) {
            continue;
        }
        set_data(&item, format, &representation.data)?;
    }

    if image_types.is_empty() {
        let bytes = std::fs::read(path).with_context(|| format!("read copied image {}", path.display()))?;
        if let Some(format) = detect_image_format(path, &bytes) {
            set_data(&item, format, &bytes)?;
        }
    }

    pasteboard.clearContents();
    let objects = NSArray::from_retained_slice(&[ProtocolObject::from_retained(item)]);
    if !pasteboard.writeObjects(&objects) {
        bail!("publish file and image to macOS pasteboard");
    }
    Ok(())
}

fn set_data(item: &NSPasteboardItem, format: &str, bytes: &[u8]) -> Result<()> {
    let data = NSData::with_bytes(bytes);
    if !item.setData_forType(&data, &NSString::from_str(format)) {
        bail!("publish {format} to macOS pasteboard");
    }
    Ok(())
}

fn native_image_format(format: &str) -> Option<&'static str> {
    match format {
        "public.png" | "image/png" => Some("public.png"),
        "public.jpeg" | "image/jpeg" | "image/jpg" => Some("public.jpeg"),
        "public.tiff" | "image/tiff" => Some("public.tiff"),
        "com.compuserve.gif" | "image/gif" => Some("com.compuserve.gif"),
        "public.heic" | "image/heic" => Some("public.heic"),
        "public.heif" | "image/heif" => Some("public.heif"),
        "org.webmproject.webp" | "image/webp" => Some("org.webmproject.webp"),
        _ => None,
    }
}

fn detect_image_format(path: &Path, bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("public.png");
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("public.jpeg");
    }
    if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        return Some("public.tiff");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("com.compuserve.gif");
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("org.webmproject.webp");
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        return match &bytes[8..12] {
            b"heic" | b"heix" | b"hevc" | b"hevx" => Some("public.heic"),
            b"mif1" | b"msf1" => Some("public.heif"),
            _ => None,
        };
    }

    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => Some("public.png"),
        Some("jpg" | "jpeg") => Some("public.jpeg"),
        Some("tif" | "tiff") => Some("public.tiff"),
        Some("gif") => Some("com.compuserve.gif"),
        Some("heic") => Some("public.heic"),
        Some("heif") => Some("public.heif"),
        Some("webp") => Some("org.webmproject.webp"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publishes_a_file_url_and_image_on_the_same_pasteboard_item() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("message.png");
        let png = b"\x89PNG\r\n\x1a\nfixture";
        std::fs::write(&path, png).unwrap();
        let pasteboard = NSPasteboard::pasteboardWithUniqueName();

        publish_single_file_to(&pasteboard, &path, &[]).unwrap();

        let items = pasteboard.pasteboardItems().unwrap();
        assert_eq!(items.len(), 1);
        let item = items.iter().next().unwrap();
        assert_eq!(
            item.stringForType(&NSString::from_str("public.file-url"))
                .unwrap()
                .to_string(),
            filebundle::path_to_uri(&path)
        );
        assert_eq!(
            item.dataForType(&NSString::from_str("public.png"))
                .unwrap()
                .to_vec(),
            png
        );
    }

    #[test]
    fn recognizes_common_image_file_signatures() {
        assert_eq!(
            detect_image_format(Path::new("attachment"), b"\xff\xd8\xffbody"),
            Some("public.jpeg")
        );
        assert_eq!(
            detect_image_format(Path::new("attachment"), b"\0\0\0\x18ftypheicbody"),
            Some("public.heic")
        );
        assert_eq!(
            detect_image_format(Path::new("attachment.webp"), b"not enough bytes"),
            Some("org.webmproject.webp")
        );
    }
}
