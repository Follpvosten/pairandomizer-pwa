use core::fmt;

use anyhow::Result;
use gloo::net::http::Request;
use itertools::Itertools;
use rand::prelude::SliceRandom;
use serde::Deserialize;
use unic_langid::LanguageIdentifier;

use crate::settings::Settings;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Index {
    pub server_name: String,
    pub comment: String,
    pub scenarios: Vec<ScenarioMeta>,
    #[serde(default)]
    pub beta: Option<ScenarioMeta>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedIndex {
    pub inner: Index,
    pub scenarios: Vec<(ScenarioMeta, Scenario)>,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ScenarioMeta {
    pub name: String,
    pub lang: Option<String>,
    pub filename: String,
}
impl Index {
    pub async fn load(server_url: &str) -> Result<LoadedIndex> {
        let me = Self::fetch(server_url).await?;
        me.load_scenarios(server_url).await
    }
    pub async fn fetch(server_url: &str) -> Result<Self> {
        Ok(Request::get(&format!("{server_url}/res/json/index.json"))
            .send()
            .await?
            .json()
            .await?)
    }
    pub async fn load_scenarios(self, server_url: &str) -> Result<LoadedIndex> {
        let mut scenarios =
            Vec::with_capacity(self.scenarios.len() + if self.beta.is_some() { 1 } else { 0 });
        for meta in self.scenarios.iter().chain(&self.beta).cloned() {
            let scenario: Scenario =
                Request::get(&format!("{server_url}/res/json/{}", meta.filename))
                    .send()
                    .await?
                    .json()
                    .await?;
            scenarios.push((meta, scenario));
        }
        Ok(LoadedIndex {
            inner: self,
            scenarios,
        })
    }
}
impl LoadedIndex {
    /// Picks a random scenario.
    ///
    /// If settings.scenario_index is Some and the stored index exists, that
    /// scenario will be returned.
    ///
    /// If settings.ignore_language is true, it's picked out of all available
    /// scenarios known to the index.
    ///
    /// When it is false, this method collects a list of native language
    /// scenarios and if there's more than 0, it returns a random one of these.
    ///
    /// If there's no native language scenarios, it behaves as if
    /// ignore_language is false.
    pub fn pick_scenario<R: rand::Rng + ?Sized>(
        &self,
        rng: &mut R,
        settings: &Settings,
        lang_id: &LanguageIdentifier,
    ) -> &(ScenarioMeta, Scenario) {
        if let Some(index) = settings.scenario_index {
            if let Some(scenario) = self.scenarios.get(index) {
                return scenario;
            }
        }
        if settings.ignore_language {
            self.scenarios.choose(rng).unwrap()
        } else {
            let native_scenarios = self
                .scenarios
                .iter()
                .filter(|(s, _)| s.lang.as_deref() == Some(lang_id.language.as_str()))
                .collect_vec();
            if native_scenarios.is_empty() {
                self.scenarios.choose(rng).unwrap()
            } else {
                native_scenarios.choose(rng).unwrap()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Scenario {
    pub scenes: Vec<Scene>,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Scene {
    pub name: String,
    pub messages: Vec<String>,
}
impl Scene {
    pub fn randomize<'a, 'b>(&'a self, names: &'b [String]) -> Vec<RandomizedMsg<'a, 'b>> {
        let mut result = Vec::with_capacity(names.len() / 2);
        let name_pairs = names
            .iter()
            .sorted_by_key(|_| rand::random::<u8>())
            .chunks(2);
        let name_pairs =
            name_pairs
                .into_iter()
                .filter_map(|mut chunk| match (chunk.next(), chunk.next()) {
                    (Some(name1), Some(name2)) => Some((name1, name2)),
                    _ => None,
                });
        let mut rng = rand::thread_rng();
        for names in name_pairs {
            result.push(RandomizedMsg {
                message: self.messages.choose(&mut rng).unwrap(),
                names,
            });
        }
        result
    }
}

pub struct RandomizedMsg<'scene, 'names> {
    pub message: &'scene String,
    pub names: (&'names String, &'names String),
}

impl<'a, 'b> fmt::Display for RandomizedMsg<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.message
                .replace("%1$s", self.names.0)
                .replace("%2$s", self.names.1)
        )
    }
}
