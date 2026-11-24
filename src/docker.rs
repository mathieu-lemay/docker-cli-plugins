use anyhow::{bail, Result};
use bollard::query_parameters::ListContainersOptionsBuilder;
use bollard::Docker;
use skim::options::SkimOptionsBuilder;
use skim::Skim;

use crate::error::{NoContainersError, NoSelectionError, Resource};

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

    let mut builder = SkimOptionsBuilder::default();

    builder.height("50%");
    builder.multi(options.multi);
    builder.select_1(true);
    builder.reverse(true);

    if let Some(q) = options.query {
        builder.query(q);
    }

    if options.pre_select {
        builder.pre_select_n(containers.len());
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
