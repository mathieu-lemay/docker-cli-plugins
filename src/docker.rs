use anyhow::{bail, Result};
use bollard::query_parameters::{ListContainersOptionsBuilder, ListImagesOptions};
use bollard::Docker;

use crate::error::{NoContainersError, NoImagesError};
use crate::fzf::{fuzzy_search, SearchOptions};

#[derive(Default)]
pub struct FindContainersOptions {
    pub query: Option<String>,
    pub all: bool,
    pub multi: bool,
    pub pre_select: bool,
}

impl From<FindContainersOptions> for SearchOptions {
    fn from(options: FindContainersOptions) -> Self {
        SearchOptions {
            query: options.query,
            multi: options.multi,
            pre_select: options.pre_select,
        }
    }
}

pub async fn find_containers(d: &Docker, options: FindContainersOptions) -> Result<Vec<String>> {
    let containers = get_container_names(d, options.all).await?;

    fuzzy_search(containers, options)
}

pub async fn get_container_names(d: &Docker, all: bool) -> Result<Vec<String>> {
    let opts = ListContainersOptionsBuilder::new().all(all).build();

    let containers = d.list_containers(Some(opts)).await?;
    if containers.is_empty() {
        bail!(NoContainersError);
    }

    let names = containers.iter().fold(Vec::new(), |mut v, i| {
        if let Some(names) = &i.names {
            for n in names {
                v.push(n.trim_start_matches("/").to_string());
            }
        };

        v
    });

    Ok(names)
}

#[derive(Default)]
pub struct FindImagesOptions {
    pub query: Option<String>,
    pub multi: bool,
    pub pre_select: bool,
}

impl From<FindImagesOptions> for SearchOptions {
    fn from(options: FindImagesOptions) -> Self {
        SearchOptions {
            query: options.query,
            multi: options.multi,
            pre_select: options.pre_select,
        }
    }
}

pub async fn find_images(d: &Docker, options: FindImagesOptions) -> Result<Vec<String>> {
    let images = get_image_tags(d).await?;

    fuzzy_search(images, options)
}

async fn get_image_tags(d: &Docker) -> Result<Vec<String>> {
    let images = d.list_images(None::<ListImagesOptions>).await?;

    let tags = images.iter().fold(Vec::new(), |mut v, i| {
        for t in &i.repo_tags {
            v.push(t.trim_start_matches("/").to_string());
        }

        v
    });

    if tags.is_empty() {
        bail!(NoImagesError);
    }

    Ok(tags)
}
