use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use skim::options::SkimOptionsBuilder;
use skim::{Selector, Skim, SkimItem, SkimOptions};

use crate::error::NoSelectionError;

#[derive(Default)]
pub struct SearchOptions {
    pub query: Option<String>,
    pub multi: bool,
    pub pre_select: bool,
}

pub fn fuzzy_search(values: Vec<String>, opts: impl Into<SearchOptions>) -> Result<Vec<String>> {
    let sk_opts = build_skim_options(opts)?;

    let output = Skim::run_items(sk_opts, values).map_err(|e| anyhow!(e))?;

    if output.is_abort || output.selected_items.is_empty() {
        bail!(NoSelectionError);
    }

    Ok(output
        .selected_items
        .iter()
        .map(|i| i.item.text().to_string())
        .collect())
}

struct AllItemsSelector {}
impl Selector for AllItemsSelector {
    fn should_select(&self, _: usize, _: &dyn SkimItem) -> bool {
        true
    }
}

fn build_skim_options(opts: impl Into<SearchOptions>) -> Result<SkimOptions> {
    let mut builder = SkimOptionsBuilder::default();

    let opts = opts.into();

    builder.height("50%");
    builder.select_1(true);
    builder.reverse(true);
    builder.bind(vec![
        "ctrl-a:select-all".to_string(),
        "ctrl-d:deselect-all".to_string(),
        "ctrl-t:toggle-all".to_string(),
    ]);

    builder.multi(opts.multi);

    if let Some(q) = opts.query {
        builder.query(q);

        // Only pre-select when a query was provided
        if opts.pre_select {
            builder.selector(Rc::new(AllItemsSelector {}));
        }
    }

    builder.build().map_err(|e| anyhow!(e))
}
