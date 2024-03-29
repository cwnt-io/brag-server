use std::{error::Error, path::PathBuf, slice::Iter};

use cmd_lib::{run_cmd, run_fun};
use reqwest::{header::USER_AGENT, Client};
use serde::Serialize;
use serde_json::Value;
use url::Url;

use crate::{global_vars::CARGO_PKG_VERSION, utils::repos_base_path};

use super::{
    commits::Commit,
    git_hosts::{GitHost, Host},
};

#[derive(Debug)]
#[allow(unused)]
pub struct Repo {
    pub url: Url,
    user: String,
    name: String,
    host: GitHost,
    path: PathBuf,
    git_path: PathBuf,
    curr_branch: String,
    pub commits: Vec<Commit>,
    pub user_repo_name: String,
}

fn get_value(obj: &Value, key: &str) -> Result<String, Box<dyn Error>> {
    Ok(obj.get(key).ok_or(key)?.as_str().ok_or(key)?.to_owned())
}

impl Repo {
    pub fn from(host: &Host, obj: &Value) -> Result<Self, Box<dyn Error>> {
        let name = get_value(obj, &host.host.repo_name_key())?;
        let url_str = get_value(obj, &host.host.url_key())?;
        let url = Url::parse(&url_str)?;
        let user_repo_name = get_value(obj, &host.host.user_repo_name_key())?;
        let mut path = repos_base_path();
        path.push(&user_repo_name);
        let path_str = path.to_str().unwrap();
        let git_path = path.join(".git");
        let git_path_str = git_path.to_str().unwrap();
        let wt = format!("--work-tree={}", path_str);
        let gd = format!("--git-dir={}", git_path_str);
        if !path.is_dir() {
            let url = url.as_str();
            println!("# Cloning: {}\nTo: {}", url, path_str);
            run_cmd!(git clone $url $path_str)?;
            println!("# Cloned: {}", path_str);
        }
        let curr_branch = run_fun!(git $wt $gd rev-parse --abbrev-ref HEAD)?;
        Ok(Self {
            host: host.host,
            user: host.user.clone(),
            name,
            commits: Vec::default(),
            url,
            path: path.clone(),
            git_path,
            curr_branch,
            user_repo_name,
        })
    }
    pub fn pull_commits(&self) -> Result<(), Box<dyn Error>> {
        let path_str = self.path.to_str().unwrap();
        let git_path_str = self.git_path.to_str().unwrap();
        let wt = format!("--work-tree={}", path_str);
        let gd = format!("--git-dir={}", git_path_str);
        let branch = &self.curr_branch;
        println!(
            "# Pulling!\ngit_path: {}\n - current_branch: {}",
            git_path_str, branch
        );
        run_cmd!(git $wt $gd pull origin $branch)?;
        Ok(())
    }
    pub fn set_commits(&mut self) -> Result<(), Box<dyn Error>> {
        println!("(set_commits)");
        let path_str = self.path.to_str().unwrap();
        let git_path_str = self.git_path.to_str().unwrap();
        let wt = format!("--work-tree={}", path_str);
        let gd = format!("--git-dir={}", git_path_str);
        let n_commits = run_fun!(git $wt $gd rev-list --count HEAD)?;
        println!("n_commits: {}", n_commits);
        if n_commits.parse::<i32>().unwrap() == 1 {
            // commit-json script yet can't handle parsing a repo that has
            // only one commit
            return Ok(());
        }
        let branch = &self.curr_branch;
        println!(
            "# Setting up commits !\n - path: {}\n - git_path: {}\n - current_branch: {}",
            path_str, git_path_str, branch
        );
        let json_str = run_fun!(commit-json $path_str $git_path_str HEAD)?;
        self.commits = serde_json::from_str(&json_str)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Repositories(Vec<Repo>);

impl Repositories {
    pub async fn from(hosts: &Vec<Host>) -> Result<Self, Box<dyn Error>> {
        let client = Client::new();
        let mut repos = vec![];
        for host in hosts {
            let api_repos_url = host.host.api_repos_url(&host.user);
            let json_str = client
                .get(api_repos_url)
                .header(USER_AGENT, format!("brag-server/{}", CARGO_PKG_VERSION))
                .send()
                .await?
                .text()
                .await?;
            let json_repos: Vec<Value> = serde_json::from_str(&json_str)?;
            for jrepo in &json_repos {
                let repo = Repo::from(host, jrepo)?;
                repos.push(repo);
            }
        }
        Ok(Self(repos))
    }
    pub fn iter(&self) -> Iter<'_, Repo> {
        self.0.iter()
    }
    pub fn pull_all(&self) -> Result<(), Box<dyn Error>> {
        for repo in self.0.iter() {
            repo.pull_commits()?;
        }
        Ok(())
    }
    pub fn set_all_commits(&mut self) -> Result<(), Box<dyn Error>> {
        for repo in self.0.iter_mut() {
            repo.set_commits()?;
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct RepoResp {
    name: String,
    user: String,
    full_name: String,
}

impl RepoResp {
    pub fn from_full_name(full_name: &str) -> Self {
        let v: Vec<&str> = full_name.split('/').collect();
        let full_name = full_name.to_owned();
        let user = v.first().unwrap_or(&"").to_string();
        let name = v.get(1).unwrap_or(&"").to_string();
        Self {
            full_name,
            user,
            name,
        }
    }
}
