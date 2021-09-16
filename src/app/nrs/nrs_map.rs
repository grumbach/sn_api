// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{
    app::{
        fetch::{ContentType, DataType},
        Safe,
    },
    Error, Result, XorUrl,
};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) type SubName = String;

/// Mapping SubNames to XorUrls
/// For a given Top Name : "example"
///
/// | SubName Key   | Full Name        | XorUrl Value |
/// |---------------|------------------|--------------|
/// | ""            | "example"        | "safe://eg1" |
/// | "sub"         | "sub.example"    | "safe://eg2" |
/// | "sub.sub"     | "sub.sub.example"| "safe://eg3" |
///
#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct NrsMap {
    pub map: BTreeMap<SubName, XorUrl>,
}

impl NrsMap {
    pub fn resolve_for_subnames(&self, sub_names: &[SubName]) -> Result<XorUrl> {
        debug!("NRS: Attempting to resolve for subnames {:?}", sub_names);
        let sub_names_str = sub_names_vec_to_str(sub_names);
        self.get_link_for(&sub_names_str)
    }

    pub fn get_default_link(&self) -> Result<XorUrl> {
        debug!("Attempting to get default link with NRS....");
        self.get_link_for("")
    }

    pub fn nrs_map_remove_subname(&mut self, name: &str) -> Result<String> {
        info!("Removing sub name \"{}\" from NRS map", name);
        match self.map.remove(name) {
            Some(link) => Ok(link),
            None => Err(Error::ContentError(
                "Sub name not found in NRS Map Container".to_string(),
            )),
        }
    }

    pub fn update(&mut self, name: &str, link: &str) -> Result<String> {
        info!("Updating NRS map for: {}", name);
        // NRS resolver doesn't allow unversioned links
        validate_nrs_link(link)?;
        let subname = parse_out_subnames(name);
        self.map.insert(subname, link.to_owned());
        Ok(link.to_string())
    }

    pub fn get_link_for(&self, sub_name: &str) -> Result<XorUrl> {
        match self.map.get(sub_name) {
            Some(link) => {
                debug!("NRS: Subname resolution is: {} => {}", sub_name, link);
                Ok(link.to_owned())
            }
            None => {
                debug!("NRS: No link found for subname(s): {}", sub_name);
                Err(Error::ContentError(format!(
                    "Link not found in NRS Map Container for: {}",
                    sub_name
                )))
            }
        }
    }
}

fn sub_names_vec_to_str(sub_names: &[SubName]) -> String {
    if !sub_names.is_empty() {
        let length = sub_names.len() - 1;
        sub_names
            .iter()
            .enumerate()
            .map(|(i, n)| {
                if i < length {
                    format!("{}.", n)
                } else {
                    n.to_string()
                }
            })
            .collect()
    } else {
        "".to_string()
    }
}

/// removes top name from a given name
/// "sub.sub.topname" -> "sub.sub"
/// "sub.cooltopname" -> "sub"
/// "lonetopname" -> ""
fn parse_out_subnames(name: &str) -> String {
    let sanitized_name = str::replace(name, "safe://", "");
    let mut parts = sanitized_name.split('.');
    // pop out the topname (last part)
    let _ = parts.next_back();
    parts.collect::<Vec<&str>>().join(".")
}

pub(crate) fn validate_nrs_link(link: &str) -> Result<()> {
    let link_encoder = Safe::parse_url(link)?;
    if link_encoder.content_version().is_none() {
        let content_type = link_encoder.content_type();
        let data_type = link_encoder.data_type();
        if content_type == ContentType::FilesContainer
            || content_type == ContentType::NrsMapContainer
        {
            return Err(Error::InvalidInput(format!(
                "The linked content ({}) is versionable, therefore NRS requires the link to specify a hash: \"{}\"",
                content_type, link
            )));
        } else if data_type == DataType::Register {
            return Err(Error::InvalidInput(format!(
                "The linked content ({}) is versionable, therefore NRS requires the link to specify a hash: \"{}\"",
                data_type, link
            )));
        }
    }

    Ok(())
}
