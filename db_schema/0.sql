CREATE TABLE schema_changelog (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    version INTEGER NOT NULL UNIQUE,
    date TIMESTAMP WITHOUT TIME ZONE DEFAULT NOW()
);

INSERT INTO schema_changelog (version) VALUES (0);

CREATE TABLE calendars (
    id INTEGER GENERATED ALWAYS AS IDENTITY,

    CONSTRAINT calendar_pkey PRIMARY KEY (id)
);

CREATE TABLE events (
    id integer GENERATED ALWAYS AS IDENTITY,
    parent_event_id integer,
    start_date date NOT NULL,
    start_time time without time zone,
    end_date date NOT NULL,
    end_time time without time zone,
    rrule text,
    exdates date[],
    rdates date[],
    calendar_id integer NOT NULL,

    CONSTRAINT end_later_than_start CHECK (((start_date + start_time) < (end_date + end_time))),
    CONSTRAINT no_exdates_rdates_if_no_rrule CHECK (((rrule IS NOT NULL) OR ((exdates IS NULL) AND (rdates IS NULL)))),
    CONSTRAINT no_recurrent_child CHECK ((NOT ((parent_event_id IS NOT NULL) AND (rrule IS NOT NULL)))),
    CONSTRAINT no_rrule_exdates_rdates_when_child CHECK (((parent_event_id IS NULL) OR ((rrule IS NULL) AND (exdates IS NULL) AND (rdates IS NULL)))),
    CONSTRAINT start_and_end_times CHECK (((start_time IS NULL) = (end_time IS NULL))),

    CONSTRAINT events_pkey PRIMARY KEY (id),
    CONSTRAINT fk_calendar_id FOREIGN KEY (calendar_id) REFERENCES calendars(id),
    CONSTRAINT fk_parent_event_id FOREIGN KEY (parent_event_id) REFERENCES events(id)
);
