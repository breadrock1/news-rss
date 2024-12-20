-- Add up migration script here

CREATE TABLE IF NOT EXISTS rss_sources(
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    link TEXT NOT NULL,
    run_at_launch BOOL NOT NULL DEFAULT false,
    max_retries INTEGER NOT NULL DEFAULT 3,
    timeout INTEGER NOT NULL DEFAULT 100,
    interval_secs INTEGER NOT NULL DEFAULT 3600
);

INSERT INTO rss_sources(name, link, run_at_launch)
VALUES ('NDTV World News', 'https://feeds.feedburner.com/ndtvnews-world-news', true);

INSERT INTO rss_sources(name, link, run_at_launch)
VALUES ('Sky World News', 'https://feeds.skynews.com/feeds/rss/world.xml', false);
