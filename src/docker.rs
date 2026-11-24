use anyhow::{bail, Result};
use bollard::query_parameters::{ListContainersOptionsBuilder, ListImagesOptions};
use bollard::Docker;
use skim::options::SkimOptionsBuilder;
use skim::Skim;

use crate::error::{NoContainersError, NoImagesError, NoSelectionError, Resource};

#[derive(Default)]
pub struct FindContainersOptions<'a> {
    pub query: Option<&'a str>,
    pub all: bool,
    pub multi: bool,
    pub pre_select: bool,
}

pub async fn find_containers<'a>(
    d: &Docker,
    options: FindContainersOptions<'a>,
) -> Result<Vec<String>> {
    let containers = get_container_names(d, options.all).await?;

    let mut builder = skim_options();

    builder.multi(options.multi);

    if let Some(q) = options.query {
        builder.query(q);

        // Only pre-select when a query was provided
        if options.pre_select {
            builder.pre_select_n(containers.len());
        }
    }

    let options = builder.build()?;

    let result = Skim::run_items(options, containers).map(|out| out.selected_items);

    match result {
        Ok(i) => {
            if i.is_empty() {
                bail!(NoSelectionError(Resource::Container));
            }

            Ok(i.iter().map(|i| i.item.text().to_string()).collect())
        }
        Err(e) => bail!(e),
    }
}

async fn get_container_names(d: &Docker, all: bool) -> Result<Vec<String>> {
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
pub struct FindImagesOptions<'a> {
    pub query: Option<&'a str>,
    pub multi: bool,
    pub pre_select: bool,
}

pub async fn find_images<'a>(d: &Docker, options: FindImagesOptions<'a>) -> Result<Vec<String>> {
    let images = get_image_tags(d).await?;

    let mut builder = skim_options();

    builder.multi(options.multi);

    if let Some(q) = options.query {
        builder.query(q);

        // Only pre-select when a query was provided
        if options.pre_select {
            builder.pre_select_n(images.len());
        }
    }

    let options = builder.build()?;

    let result = Skim::run_items(options, images).map(|out| out.selected_items);

    match result {
        Ok(i) => {
            if i.is_empty() {
                bail!(NoSelectionError(Resource::Image));
            }

            Ok(i.iter().map(|i| i.item.text().to_string()).collect())
        }
        Err(e) => bail!(e),
    }
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

fn skim_options() -> SkimOptionsBuilder {
    let mut builder = SkimOptionsBuilder::default();

    builder.height("50%");
    builder.select_1(true);
    builder.reverse(true);
    builder.bind(vec![
        "ctrl-a:select-all".to_string(),
        "ctrl-d:deselect-all".to_string(),
        "ctrl-t:toggle-all".to_string(),
    ]);

    builder
}
