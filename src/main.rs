use csv::Reader;
use nlp::phonetics::metaphone::double_metaphone::double_metaphone;
use once_cell::sync::Lazy;
use regex::Regex;
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::{error::Error, fs::File};

mod dft;

static KEYWORD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-z]+|[0-9]+").unwrap());

static CURRENT_FULL_YEAR: &str = "2024";

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Link {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
struct Make {
    pub name: String,
    pub slug: String,
    pub generic_models: BTreeSet<Link>,
    #[serde(flatten)]
    pub stats: Stats,
}

impl Make {
    fn new(name: &str, slug: &str) -> Self {
        Make {
            name: name.to_string(),
            slug: slug.to_string(),
            generic_models: BTreeSet::new(),
            stats: Stats::new(),
        }
    }

    fn link(&self) -> Link {
        Link {
            name: self.name.clone(),
            slug: self.slug.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct GenericModel {
    pub name: String,
    pub slug: String,
    pub make: Link,
    pub models: BTreeSet<Link>,
    #[serde(flatten)]
    pub stats: Stats,
}

impl GenericModel {
    fn new(make: &Make, name: &str, slug: &str) -> Self {
        GenericModel {
            name: name.to_string(),
            slug: slug.to_string(),
            make: make.link(),
            models: BTreeSet::new(),
            stats: Stats::new(),
        }
    }

    fn link(&self) -> Link {
        Link {
            name: self.name.clone(),
            slug: self.slug.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Model {
    pub name: String,
    pub slug: String,
    pub make: Link,
    pub generic_model: Link,
    #[serde(flatten)]
    pub stats: Stats,
    index: usize,
}

impl Model {
    fn new(
        make: &Make,
        generic_model: &GenericModel,
        name: &str,
        slug: &str,
        index: usize,
    ) -> Self {
        Model {
            name: name.to_string(),
            slug: slug.to_string(),
            make: make.link(),
            generic_model: generic_model.link(),
            stats: Stats::new(),
            index,
        }
    }

    fn link(&self) -> Link {
        Link {
            name: self.name.clone(),
            slug: self.slug.clone(),
        }
    }

    fn full_name(&self) -> String {
        [self.make.name.as_str(), self.name.as_str()].join(" ")
    }

    fn keywords(&self) -> HashSet<String> {
        KEYWORD_RE
            .find_iter(&self.full_name().to_ascii_lowercase())
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

#[derive(Debug, Serialize)]
struct Stats {
    pub quarterly_licensed: BTreeMap<String, i32>,
    pub quarterly_sorn: BTreeMap<String, i32>,

    pub first_reg_licensed: BTreeMap<String, i32>,
    pub first_reg_sorn: BTreeMap<String, i32>,

    pub manufacture_licensed: BTreeMap<String, i32>,
    pub manufacture_sorn: BTreeMap<String, i32>,

    pub new_reg: BTreeMap<String, i32>,

    pub petrol_licensed: BTreeMap<String, i32>,
    pub petrol_sorn: BTreeMap<String, i32>,

    pub diesel_licensed: BTreeMap<String, i32>,
    pub diesel_sorn: BTreeMap<String, i32>,

    pub other_licensed: BTreeMap<String, i32>,
    pub other_sorn: BTreeMap<String, i32>,
}

impl Stats {
    fn new() -> Stats {
        Stats {
            quarterly_licensed: BTreeMap::new(),
            quarterly_sorn: BTreeMap::new(),
            first_reg_licensed: BTreeMap::new(),
            first_reg_sorn: BTreeMap::new(),
            manufacture_licensed: BTreeMap::new(),
            manufacture_sorn: BTreeMap::new(),
            new_reg: BTreeMap::new(),
            petrol_licensed: BTreeMap::new(),
            petrol_sorn: BTreeMap::new(),
            diesel_licensed: BTreeMap::new(),
            diesel_sorn: BTreeMap::new(),
            other_licensed: BTreeMap::new(),
            other_sorn: BTreeMap::new(),
        }
    }

    fn merge_veh0120_gb(&mut self, row: &dft::Veh0120) -> Result<(), Box<dyn Error>> {
        for (k, v) in row.extra.iter() {
            if *v <= 0 {
                continue;
            }
            // Only use GB values from pre-2014Q3.
            if "2014Q3".gt(k) {
                let k2 = k.replace("Q", " q");
                match &row.licence_status {
                    dft::LicenceStatus::Licensed => {
                        *self.quarterly_licensed.entry(k2).or_insert(0) += v
                    }
                    dft::LicenceStatus::SORN => *self.quarterly_sorn.entry(k2).or_insert(0) += v,
                }
            }
        }
        Ok(())
    }

    fn merge_veh0120_uk(&mut self, row: &dft::Veh0120) -> Result<(), Box<dyn Error>> {
        for (k, v) in row.extra.iter() {
            if *v <= 0 {
                continue;
            }
            let k2 = k.replace("Q", " q");
            match &row.licence_status {
                dft::LicenceStatus::Licensed => {
                    *self.quarterly_licensed.entry(k2).or_insert(0) += v
                }
                dft::LicenceStatus::SORN => *self.quarterly_sorn.entry(k2).or_insert(0) += v,
            }
        }
        Ok(())
    }

    fn merge_veh0160_gb(&mut self, row: &dft::Veh0160) -> Result<(), Box<dyn Error>> {
        for (k, v) in row.extra.iter() {
            if *v <= 0 {
                continue;
            }
            // Only use GB values from pre-2014Q3.
            if "2014Q3".gt(k) {
                let k2 = k.replace("Q", " q");
                *self.new_reg.entry(k2).or_insert(0) += v;
            }
        }
        Ok(())
    }

    fn merge_veh0160_uk(&mut self, row: &dft::Veh0160) -> Result<(), Box<dyn Error>> {
        for (k, v) in row.extra.iter() {
            if *v <= 0 {
                continue;
            }
            let k2 = k.replace("Q", " q");
            *self.new_reg.entry(k2).or_insert(0) += v;
        }
        Ok(())
    }

    fn merge_veh0124(&mut self, row: &dft::Veh0124) -> Result<(), Box<dyn Error>> {
        // TODO: Yearly breakdowns, not just current year.
        if let Some(dft::OptionalNumber::Count(n)) = row.extra.get(CURRENT_FULL_YEAR) {
            if *n <= 0 {
                return Ok(());
            }
            let mk = match row.manufactured {
                dft::OptionalNumber::Count(y) => y.to_string(),
                _ => "Unknown".to_string(),
            };
            let fk = match row.first_used {
                dft::OptionalNumber::Count(y) => y.to_string(),
                _ => "Unknown".to_string(),
            };
            match &row.licence_status {
                dft::LicenceStatus::Licensed => {
                    *self.manufacture_licensed.entry(mk).or_insert(0) += n;
                    *self.first_reg_licensed.entry(fk).or_insert(0) += n;
                }
                dft::LicenceStatus::SORN => {
                    *self.manufacture_sorn.entry(mk).or_insert(0) += n;
                    *self.first_reg_sorn.entry(fk).or_insert(0) += n;
                }
            }
        }
        Ok(())
    }

    fn merge_veh0220(&mut self, row: &dft::Veh0220) -> Result<(), Box<dyn Error>> {
        // TODO: Yearly breakdowns, not just current year.
        if let Some(n) = row.extra.get(CURRENT_FULL_YEAR) {
            if *n <= 0 {
                return Ok(());
            }
            let engine_size = if row.engine_size_desc == "[z]" || row.engine_size_desc == "[x]" {
                "Unknown"
            } else {
                &row.engine_size_desc
            }
            .to_string();
            match (&row.licence_status, &row.fuel) {
                (dft::LicenceStatus::Licensed, dft::FuelType::Petrol) => {
                    *self.petrol_licensed.entry(engine_size).or_insert(0) += n;
                }
                (dft::LicenceStatus::SORN, dft::FuelType::Petrol) => {
                    *self.petrol_sorn.entry(engine_size).or_insert(0) += n;
                }
                (dft::LicenceStatus::Licensed, dft::FuelType::Diesel) => {
                    *self.diesel_licensed.entry(engine_size).or_insert(0) += n;
                }
                (dft::LicenceStatus::SORN, dft::FuelType::Diesel) => {
                    *self.diesel_sorn.entry(engine_size).or_insert(0) += n;
                }
                (dft::LicenceStatus::Licensed, _) => {
                    *self.other_licensed.entry(engine_size).or_insert(0) += n;
                }
                (dft::LicenceStatus::SORN, _) => {
                    *self.other_sorn.entry(engine_size).or_insert(0) += n;
                }
            }
        }
        Ok(())
    }
}

fn slugify(parts: &[&str]) -> Result<String, String> {
    if !parts.iter().all(|s| s.chars().all(|c| c.is_ascii())) {
        Err(format!("Invalid characters in name: {:?}", parts))
    } else {
        Ok(parts
            .iter()
            .map(|p| p.to_lowercase().replace(&[' ', '/'], "_"))
            .collect::<Vec<_>>()
            .join("_"))
    }
}

fn read_table<T, F>(filename: &str, mut callback: F) -> Result<(), Box<dyn Error>>
where
    T: DeserializeOwned,
    F: FnMut(T) -> Result<(), Box<dyn Error>>,
{
    let reader = Reader::from_reader(File::open(filename)?);
    for result in reader.into_deserialize() {
        callback(result?)?;
    }
    Ok(())
}

fn to_blob(indices: &BTreeSet<usize>) -> Vec<u8> {
    let mut ret = Vec::with_capacity(indices.len() * 4);
    for index in indices {
        ret.push((index & 255) as u8);
        ret.push(((index >> 8) & 255) as u8);
        ret.push(((index >> 16) & 255) as u8);
        ret.push(((index >> 24) & 255) as u8);
    }
    ret
}

struct Index {
    makes: BTreeMap<String, Make>,
    generic_models: BTreeMap<String, GenericModel>,
    models: BTreeMap<String, Model>,
    keywords: BTreeMap<String, BTreeSet<usize>>,
    metaphones: BTreeMap<String, BTreeSet<String>>,
}

impl Index {
    fn new() -> Index {
        Index {
            models: BTreeMap::new(),
            generic_models: BTreeMap::new(),
            makes: BTreeMap::new(),
            keywords: BTreeMap::new(),
            metaphones: BTreeMap::new(),
        }
    }

    fn insert<R>(
        &mut self,
        row: R,
        update: fn(&mut Stats, &R) -> Result<(), Box<dyn Error>>,
    ) -> Result<(), Box<dyn Error>>
    where
        R: dft::HasIdentity + Clone,
    {
        let dft::VehicleIdentity {
            make: make_name,
            generic_model: generic_model_name,
            model: model_name,
            ..
        } = row.identity();
        let make_slug = slugify(&[&make_name])?;
        let generic_model_slug = slugify(&[&make_name, &generic_model_name])?;
        let model_slug = slugify(&[&make_name, &model_name])?;
        let make = self
            .makes
            .entry(make_slug.clone())
            .or_insert_with(|| Make::new(make_name, &make_slug));
        let generic_model = self
            .generic_models
            .entry(generic_model_slug.clone())
            .or_insert_with(|| GenericModel::new(make, generic_model_name, &generic_model_slug));
        make.generic_models.insert(generic_model.link());
        let next_model_index = self.models.len();
        let model = self.models.entry(model_slug.clone()).or_insert_with(|| {
            Model::new(
                make,
                generic_model,
                model_name,
                &model_slug,
                next_model_index,
            )
        });
        generic_model.models.insert(model.link());
        update(&mut make.stats, &row)?;
        update(&mut generic_model.stats, &row)?;
        update(&mut model.stats, &row)?;
        for word in model.keywords() {
            self.keywords
                .entry(word.clone())
                .or_insert_with(|| BTreeSet::new())
                .insert(model.index);
            if word.len() > 4 {
                if let Some(res) = double_metaphone(&word) {
                    self.metaphones
                        .entry(res.primary)
                        .or_insert_with(|| BTreeSet::new())
                        .insert(word.clone());
                    self.metaphones
                        .entry(res.alternate)
                        .or_insert_with(|| BTreeSet::new())
                        .insert(word);
                }
            }
        }

        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn Error>> {
        let _ = std::fs::remove_file("howmanyleft.sqlite3");
        let db = Connection::open("howmanyleft.sqlite3")?;
        db.execute_batch(
            "PRAGMA journal_mode = OFF;
             PRAGMA synchronous = 0;
             PRAGMA cache_size = 1000000;
             PRAGMA locking_mode = EXCLUSIVE;
             PRAGMA temp_store = MEMORY;
             CREATE TABLE makes (slug VARCHAR(255) PRIMARY KEY, name VARCHAR(255), json TEXT);
             CREATE TABLE generic_models (slug VARCHAR(255) PRIMARY KEY, json TEXT);
             CREATE TABLE models (slug VARCHAR(255) PRIMARY KEY, id UNSIGNED INTEGER UNIQUE, json TEXT);
             CREATE TABLE keywords (keyword VARCHAR(255) PRIMARY KEY, bytes BLOB);
             CREATE TABLE metaphones (metaphone VARCHAR(255) PRIMARY KEY, data TEXT);",
        )?;
        {
            let mut stmt = db.prepare("INSERT INTO makes VALUES (?1, ?2, ?3)")?;
            for make in self.makes.values() {
                stmt.execute([&make.slug, &make.name, &serde_json::to_string(&make)?])?;
            }
        }
        {
            let mut stmt = db.prepare("INSERT INTO generic_models VALUES (?1, ?2)")?;
            for generic_model in self.generic_models.values() {
                stmt.execute([&generic_model.slug, &serde_json::to_string(&generic_model)?])?;
            }
        }
        {
            let mut stmt = db.prepare("INSERT INTO models VALUES (?1, ?2, ?3)")?;
            for model in self.models.values() {
                stmt.execute((&model.slug, &model.index, &serde_json::to_string(&model)?))?;
            }
        }
        {
            let mut stmt = db.prepare("INSERT INTO keywords VALUES (?1, ?2)")?;
            for (word, indices) in &self.keywords {
                stmt.execute((word, to_blob(indices)))?;
            }
        }
        {
            let mut stmt = db.prepare("INSERT INTO metaphones VALUES (?1, ?2)")?;
            for (metaphone, words) in &self.metaphones {
                stmt.execute((
                    metaphone,
                    words.clone().into_iter().collect::<Vec<String>>().join("|"),
                ))?;
            }
        }
        Ok(())
    }
}

fn parse() -> Result<Index, Box<dyn Error>> {
    let mut index = Index::new();

    read_table("tmp/csv/df_VEH0120_GB.csv", |r| {
        index.insert(r, |s, r| s.merge_veh0120_gb(r))
    })?;
    read_table("tmp/csv/df_VEH0120_UK.csv", |r| {
        index.insert(r, |s, r| s.merge_veh0120_uk(r))
    })?;
    read_table("tmp/csv/df_VEH0124_AM.csv", |r| {
        index.insert(r, |s, r| s.merge_veh0124(r))
    })?;
    read_table("tmp/csv/df_VEH0124_NZ.csv", |r| {
        index.insert(r, |s, r| s.merge_veh0124(r))
    })?;
    read_table("tmp/csv/df_VEH0160_GB.csv", |r| {
        index.insert(r, |s, r| s.merge_veh0160_gb(r))
    })?;
    read_table("tmp/csv/df_VEH0160_UK.csv", |r| {
        index.insert(r, |s, r| s.merge_veh0160_uk(r))
    })?;
    read_table("tmp/csv/df_VEH0220.csv", |r| {
        index.insert(r, |s, r| s.merge_veh0220(r))
    })?;

    Ok(index)
}

fn main() {
    match parse() {
        Ok(index) => {
            if let Err(err) = index.save() {
                println!("Save error: {}", err);
            }
        }
        Err(err) => {
            println!("Parse error: {}", err);
        }
    }
}
