//This file is part of rubygem_api
//
//ruygem_api is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//rubygem_api is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with Foobar.  If not, see <http://www.gnu.org/licenses/>.

use log::{debug, info};
use reqwest::{StatusCode, Url};
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use failure::Fail;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Http(reqwest::Error),
    #[fail(display = "{}", _0)]
    Url(url::ParseError),
    #[fail(display = "Not found")]
    NotFound,
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::Url(e)
    }
}

pub struct SyncClient {
    client: reqwest::Client,
    base_url: Url,
}

#[derive(Deserialize, Debug)]
pub struct GemDevDeps {
    pub name: String,
    pub requirements: String,
}

#[derive(Deserialize, Debug)]
pub struct GemRunDeps {
    pub name: String,
    pub requirements: String,
}

#[derive(Deserialize, Debug)]
pub struct GemDeps {
    pub development: Option<Vec<GemDevDeps>>,
    pub runtime: Option<Vec<GemRunDeps>>,
}

#[derive(Deserialize, Debug)]
pub struct GemInfo {
    pub name: String,
    pub authors: String,
    pub version: String,
    pub info: Option<String>,
    pub licenses: Option<Vec<String>>,
    pub project_uri: String,
    pub gem_uri: String,
    pub homepage_uri: Option<String>,
    pub wiki_uri: Option<String>,
    pub documentation_uri: Option<String>,
    pub dependencies: GemDeps,
    pub sha: String,
}

impl SyncClient {
    /// Instantiate a new synchronous API client.
    ///
    /// This will fail if the underlying http client could not be created.
    pub fn new() -> Self {
        SyncClient {
            client: reqwest::Client::new(),
            base_url: Url::parse("https://rubygems.org/api/v1/gems/").unwrap(),
        }
    }

    fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, Error> {
        info!("GET {}", url);

        let mut res = {
            let res = self.client.get(url).send()?;

            if res.status() == StatusCode::NOT_FOUND {
                return Err(Error::NotFound);
            }
            res.error_for_status()?
        };

        let data: T = res.json()?;
        Ok(data)
    }

    /// Download all relevant data of a Gem from RubyGems.org
    ///
    /// Will fail if either the Gem couldn't be found or querying the API failed
    pub fn gem_info(&self, name: &str) -> Result<GemInfo, Error> {
        let url = self.base_url.join(&format!("{}.json", &name))?;
        let data: GemInfo = self.get(url)?;

        debug!("Received data from API: {:?}", data);

        let deserialized_gemdeps = GemDeps {
            development: data.dependencies.development,
            runtime: data.dependencies.runtime,
        };

        let deserialized_geminfo = GemInfo {
            name: data.name,
            version: data.version,
            authors: data.authors,
            info: data.info,
            licenses: data.licenses,
            project_uri: data.project_uri,
            gem_uri: data.gem_uri,
            homepage_uri: data.homepage_uri,
            wiki_uri: data.wiki_uri,
            documentation_uri: data.documentation_uri,
            dependencies: deserialized_gemdeps,
            sha: data.sha,
        };

        debug!("GemInfo: {:?}", deserialized_geminfo);

        Ok(deserialized_geminfo)
    }
}

#[cfg(test)]
mod test {
    use crate::SyncClient;

    #[test]
    fn test_name() {
        let client = SyncClient::new();
        let gem_info = client.gem_info("ruby-json").unwrap();
        assert!(gem_info.name.len() > 0);
    }

    #[test]
    fn test_deps() {
        let client = SyncClient::new();
        let gem_info = client.gem_info("ffi").unwrap();
        let gem_info_deps = gem_info.dependencies.development.unwrap();
        assert!(gem_info_deps.len() > 0);
    }

    #[test]
    fn test_license() {
        let client = SyncClient::new();
        let gem_info = client.gem_info("newrelic_rpm").unwrap();
        println!("{:?}", gem_info.licenses)
    }
}
