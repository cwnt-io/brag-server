-- Add up migration script here
CREATE TABLE IF NOT EXISTS commits (
    repo            	 VARCHAR(64) NOT NULL,
    hash            	 VARCHAR(40) NOT NULL,
    author_email    	 VARCHAR(64) NOT NULL,
    author_name     	 VARCHAR(64) NOT NULL,
    author_when     	 TIMESTAMPTZ DEFAULT NULL,
    committer_email 	 VARCHAR(64) DEFAULT NULL,
    committer_name  	 VARCHAR(64) DEFAULT NULL,
    committer_when  	 TIMESTAMPTZ DEFAULT NULL,
    message         	 TEXT
);

CREATE INDEX IF NOT EXISTS idx_commits_repo_authoremail ON commits (
    repo,
    author_email
);
