BEGIN TRANSACTION;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- DESCRIPTION --
-- Transforms id columns in events and calendars from auto generated integers to
-- UUIDs. New UUIDs are generated for each row and foreign keys (such as calendar_id in
-- events) are updated to reference the same row they referenced before, but now using
-- UUIDs.

-- rename id columns to id_old in calendars and events
ALTER TABLE calendars RENAME COLUMN id TO id_old;
ALTER TABLE events RENAME COLUMN id TO id_old;

-- create new id columns in calendars and events
ALTER TABLE calendars ADD COLUMN id uuid NOT NULL DEFAULT uuid_generate_v4();
ALTER TABLE events ADD COLUMN id uuid NOT NULL DEFAULT uuid_generate_v4();

-- drop events contraints
ALTER TABLE events DROP CONSTRAINT fk_calendar_id;
ALTER TABLE events DROP CONSTRAINT fk_parent_event_id;

-- remove pk from calendars and events
ALTER TABLE calendars DROP CONSTRAINT calendar_pkey;
ALTER TABLE events DROP CONSTRAINT events_pkey;

-- create pk for calendars and events
ALTER TABLE calendars ADD CONSTRAINT pk_calendars PRIMARY KEY (id);
ALTER TABLE events ADD CONSTRAINT pk_events PRIMARY KEY (id);

-- update calendar_id column
ALTER TABLE events RENAME COLUMN calendar_id TO calendar_id_old;
ALTER TABLE events ADD COLUMN calendar_id uuid;
UPDATE events SET calendar_id = calendars.id FROM calendars WHERE calendars.id_old = calendar_id_old;
ALTER TABLE events ALTER COLUMN calendar_id SET NOT NULL;
ALTER TABLE events DROP COLUMN calendar_id_old;
ALTER TABLE events ADD CONSTRAINT fk_calendar_id FOREIGN KEY (calendar_id) REFERENCES calendars(id);

-- update parent_event_id column
ALTER TABLE events RENAME COLUMN parent_event_id TO parent_event_id_old;
ALTER TABLE events ADD COLUMN parent_event_id uuid;
UPDATE events SET parent_event_id = events2.id FROM events as events2 WHERE events2.id_old = events.parent_event_id_old;
ALTER TABLE events DROP COLUMN parent_event_id_old;
ALTER TABLE events ADD CONSTRAINT fk_parent_event_id FOREIGN KEY (parent_event_id) REFERENCES events(id);

-- drop id_old columns in calendars and events
ALTER TABLE calendars DROP COLUMN id_old;
ALTER TABLE events DROP COLUMN id_old;


INSERT INTO schema_changelog (version) VALUES (2);

COMMIT TRANSACTION;