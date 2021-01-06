use anyhow::anyhow;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    utils::BoxedFuture,
};
use byteorder::LittleEndian;
use super::tga;


//use bevy::type_registry::TypeUuid;
use byteorder::ByteOrder;
use serde::Deserialize;


fn try_split(data: &[u8], idx: usize) -> Option<(&[u8], &[u8])> {
    if data.len() < idx {
        None
    } else {
        Some(data.split_at(idx))
    }
}

//#[derive(Debug, Deserialize, TypeUuid)]
//#[uuid = "1b0034cd-204a-4df7-93b9-1e0772ff1046"]
pub struct Paq;

#[derive(Default)]
pub struct Loader;

impl AssetLoader for Loader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            if !bytes.starts_with(b"paq\0") {
                return Err(anyhow!("Bad magic"))
            }
            let mut data = &bytes[4..];
            while data.len() > 0 {
                let mut split = data.splitn(2, |b| *b == 0);
                let name = String::from_utf8_lossy(
                    split.next().ok_or(anyhow!("missing name"))?,
                );
                let rest = split.next().ok_or(anyhow!("missing data"))?;
                let (length, rest) = try_split(rest, 4).ok_or(anyhow!("Broken size"))?;
                let length = LittleEndian::read_u32(length);
                let (asset, rest) = try_split(rest, length as usize).ok_or(anyhow!("Short asset"))?;

                load_context.set_labeled_asset(
                    &name,
                    LoadedAsset::new(tga::from_bytes(asset)?),
                );
                data = rest;
            }
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["paq"]
    }
}
