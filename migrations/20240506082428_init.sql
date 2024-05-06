-- Add migration script here
CREATE TABLE rss_cache (
    id INT NOT NULL AUTO_INCREMENT,
    raw_title TEXT NOT NULL,
    translated_title TEXT NOT NULL,

    PRIMARY KEY (id),
    UNIQUE KEY unique_raw_title (raw_title)
)
ENGINE = InnoDB;

