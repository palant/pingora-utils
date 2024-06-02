// Copyright 2024 Wladimir Palant
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Data structures required for `StaticFilesHandler` configuration

use module_utils::DeserializeMap;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::compression_algorithm::CompressionAlgorithm;

/// Command line options of the static files module
#[derive(Debug, Default, StructOpt)]
pub struct StaticFilesOpt {
    /// The root directory.
    #[structopt(short, long, parse(from_os_str))]
    pub root: Option<PathBuf>,

    /// Redirect /file%2e.txt to /file.txt and /dir to /dir/.
    #[structopt(long)]
    pub canonicalize_uri: Option<bool>,

    /// Index file to look for when displaying a directory. This command line flag can be specified
    /// multiple times.
    #[structopt(long)]
    pub index_file: Option<Vec<String>>,

    /// URI path of the page to display instead of the default Not Found page, e.g. /404.html
    #[structopt(long)]
    pub page_404: Option<String>,

    /// File extension to check when looking for pre-compressed versions of a file. This command
    /// line flag can be specified multiple times. Supported file extensions are gz (gzip),
    /// zz (zlib deflate), z (compress), br (Brotli), zst (Zstandard).
    #[structopt(long)]
    pub precompressed: Option<Vec<CompressionAlgorithm>>,
}

/// Configuration file settings of the static files module
#[derive(Debug, PartialEq, Eq, DeserializeMap)]
pub struct StaticFilesConf {
    /// The root directory.
    pub root: Option<PathBuf>,

    /// Redirect /file%2e.txt to /file.txt and /dir to /dir/.
    pub canonicalize_uri: bool,

    /// When `canonicalize_uri` is used, prefix redirect targets with the given string. This is
    /// useful when the static files handler is applied to a subdirectory of the actual webspace.
    pub redirect_prefix: Option<String>,

    /// List of index files to look for in a directory.
    pub index_file: Vec<String>,

    /// URI path of the page to display instead of the default Not Found page, e.g. /404.html
    pub page_404: Option<String>,

    /// List of file extensions to check when looking for pre-compressed versions of a file.
    /// Supported file extensions are gz (gzip), zz (zlib deflate), z (compress), br (Brotli),
    /// zst (Zstandard).
    pub precompressed: Vec<CompressionAlgorithm>,
}

impl StaticFilesConf {
    /// Merges the command line options into the current configuration. Any command line options
    /// present overwrite existing settings.
    pub fn merge_with_opt(&mut self, opt: StaticFilesOpt) {
        if opt.root.is_some() {
            self.root = opt.root;
        }

        if let Some(canonicalize_uri) = opt.canonicalize_uri {
            self.canonicalize_uri = canonicalize_uri;
        }

        if let Some(index_file) = opt.index_file {
            self.index_file = index_file;
        }

        if opt.page_404.is_some() {
            self.page_404 = opt.page_404;
        }

        if let Some(precompressed) = opt.precompressed {
            self.precompressed = precompressed;
        }
    }
}

impl Default for StaticFilesConf {
    fn default() -> Self {
        Self {
            root: None,
            canonicalize_uri: true,
            redirect_prefix: None,
            index_file: vec!["index.html".into()],
            page_404: None,
            precompressed: Vec::new(),
        }
    }
}
