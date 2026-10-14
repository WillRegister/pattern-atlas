use crate::{analysis::Feature, cipher::Cipher, model::Settings};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GematriaSearch {
    pub input: String,
    pub cross_cipher: bool,
    pub source_cipher: Option<Cipher>,
    pub dates: bool,
    pub geography: bool,
    pub sky: bool,
    pub data: bool,
}
impl Default for GematriaSearch {
    fn default() -> Self {
        Self {
            input: String::new(),
            cross_cipher: true,
            source_cipher: None,
            dates: true,
            geography: true,
            sky: true,
            data: true,
        }
    }
}
impl GematriaSearch {
    pub fn includes_domain(&self, domain: &str) -> bool {
        match domain {
            "Dates" => self.dates,
            "Geography" => self.geography,
            "Sky" => self.sky,
            "Data" => self.data,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputKind {
    Empty,
    Number(f64),
    Phrase,
}
#[derive(Clone, Debug)]
pub struct Target {
    pub cipher: Option<Cipher>,
    pub value: f64,
}
#[derive(Clone, Debug)]
pub struct SearchPlan {
    pub kind: InputKind,
    pub targets: Vec<Target>,
    pub cross_cipher: bool,
    enabled: Vec<Cipher>,
}

pub fn literal_number(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok().filter(|n| n.is_finite())
}
impl SearchPlan {
    pub fn new(search: &GematriaSearch, enabled: &[Cipher]) -> Result<Self> {
        let text = search.input.trim();
        ensure!(
            text.chars().count() <= 4096,
            "Use at most 4,096 characters in the live input"
        );
        let kind = if text.is_empty() {
            InputKind::Empty
        } else if let Ok(n) = text.parse::<f64>() {
            ensure!(n.is_finite(), "Enter a finite number");
            InputKind::Number(n)
        } else {
            ensure!(
                text.chars().any(|c| c.is_ascii_alphabetic()),
                "Enter a phrase containing A–Z or a plain number (for example 56 or -56.5)"
            );
            InputKind::Phrase
        };
        let selected = Cipher::ALL
            .into_iter()
            .filter(|c| {
                enabled.contains(c) && search.source_cipher.is_none_or(|source| source == *c)
            })
            .collect::<Vec<_>>();
        let targets = match kind {
            InputKind::Empty => vec![],
            InputKind::Number(value) => {
                if let Some(cipher) = search.source_cipher {
                    if selected.contains(&cipher) {
                        vec![Target {
                            cipher: Some(cipher),
                            value,
                        }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![Target {
                        cipher: None,
                        value,
                    }]
                }
            }
            InputKind::Phrase => selected
                .into_iter()
                .map(|c| Target {
                    cipher: Some(c),
                    value: c.total(text) as f64,
                })
                .collect(),
        };
        Ok(Self {
            kind,
            targets,
            cross_cipher: search.cross_cipher,
            enabled: enabled.to_vec(),
        })
    }
    pub fn matches_feature(&self, feature: &Feature, settings: &Settings) -> bool {
        self.targets
            .iter()
            .any(|t| feature.matches_value(t.value, settings))
    }
    pub fn matches_cell(&self, cipher: Option<Cipher>, raw: f64, settings: &Settings) -> bool {
        if cipher.is_some_and(|c| !self.enabled.contains(&c)) {
            return false;
        }
        let f = Feature {
            owner_id: String::new(),
            owner: String::new(),
            domain: "Gematria".into(),
            calculation: String::new(),
            unit: "value".into(),
            raw,
            explanation: String::new(),
        };
        self.targets.iter().any(|t| {
            (self.cross_cipher || cipher.is_none() || t.cipher.is_none() || cipher == t.cipher)
                && f.matches_value(t.value, settings)
        })
    }
}
