ALTER TABLE calendars ADD COLUMN IF NOT EXISTS last_modified TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW();
ALTER TABLE events ADD COLUMN IF NOT EXISTS last_modified TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW();



CREATE OR REPLACE FUNCTION update_last_modified() RETURNS TRIGGER AS $$
BEGIN
    IF(NEW != OLD) THEN
        NEW.last_modified := NOW();
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION update_last_modified() IS 'Updates the last_modified column with NOW() if any of the row''s columns have changed. Should be used in BEFORE UPDATE triggers.';




DROP TRIGGER IF EXISTS update_last_modified ON calendars;

CREATE TRIGGER update_last_modified  
BEFORE UPDATE ON calendars
FOR EACH ROW EXECUTE PROCEDURE update_last_modified();




DROP TRIGGER IF EXISTS update_last_modified ON events;

CREATE TRIGGER update_last_modified  
BEFORE UPDATE ON events
FOR EACH ROW EXECUTE PROCEDURE update_last_modified();





INSERT INTO schema_changelog (version) VALUES (1);