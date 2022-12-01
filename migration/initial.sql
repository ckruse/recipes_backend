CREATE TABLE IF NOT EXISTS ingredients (
  id bigserial NOT NULL PRIMARY KEY,
  name character varying(255) NOT NULL,
  reference character varying(255) NOT NULL,
  carbs double precision DEFAULT 0 NOT NULL,
  fat double precision DEFAULT 0 NOT NULL,
  proteins double precision DEFAULT 0 NOT NULL,
  alc double precision DEFAULT 0 NOT NULL,
  inserted_at timestamp without time zone NOT NULL,
  updated_at timestamp without time zone NOT NULL
);

-- NEXT --
CREATE TABLE IF NOT EXISTS users (
  id bigserial NOT NULL PRIMARY KEY,
  email character varying(255) NOT NULL,
  active boolean DEFAULT FALSE NOT NULL,
  encrypted_password character varying(255),
  avatar character varying(255),
  name character varying(255),
  role character varying(255),
  inserted_at timestamp without time zone NOT NULL,
  updated_at timestamp without time zone NOT NULL
);

-- NEXT --
CREATE TABLE IF NOT EXISTS ingredient_units (
  id bigserial NOT NULL PRIMARY KEY,
  ingredient_id bigint NOT NULL REFERENCES ingredients (id) ON DELETE CASCADE ON UPDATE CASCADE,
  identifier character varying(255) NOT NULL,
  base_value double precision NOT NULL,
  inserted_at timestamp without time zone NOT NULL,
  updated_at timestamp without time zone NOT NULL
);

-- NEXT --
CREATE TABLE IF NOT EXISTS recipes (
  id bigserial NOT NULL PRIMARY KEY,
  name character varying(255) NOT NULL,
  description text,
  owner_id bigint REFERENCES users (id) ON DELETE SET NULL ON UPDATE CASCADE,
  inserted_at timestamp without time zone NOT NULL,
  updated_at timestamp without time zone NOT NULL,
  image character varying(255)
);

-- NEXT --
CREATE TABLE IF NOT EXISTS tags (
  id bigserial NOT NULL PRIMARY KEY,
  name character varying(255),
  inserted_at timestamp without time zone NOT NULL,
  updated_at timestamp without time zone NOT NULL
);

-- NEXT --
CREATE TABLE IF NOT EXISTS recipes_tags (
  recipe_id bigint NOT NULL REFERENCES recipes (id) ON DELETE CASCADE ON UPDATE CASCADE,
  tag_id bigint NOT NULL REFERENCES tags (id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- NEXT --
CREATE TABLE IF NOT EXISTS steps (
  id bigserial NOT NULL PRIMARY KEY,
  recipe_id bigint NOT NULL REFERENCES recipes (id) ON DELETE CASCADE ON UPDATE CASCADE,
  "position" integer NOT NULL,
  description text,
  inserted_at timestamp without time zone NOT NULL,
  updated_at timestamp without time zone NOT NULL,
  preparation_time integer DEFAULT 0 NOT NULL,
  cooking_time integer DEFAULT 0 NOT NULL
);

-- NEXT --
CREATE TABLE IF NOT EXISTS steps_ingridients (
  id bigserial NOT NULL PRIMARY KEY,
  step_id bigint NOT NULL REFERENCES steps (id) ON DELETE CASCADE ON UPDATE CASCADE,
  ingredient_id bigint NOT NULL REFERENCES ingredients (id) ON DELETE CASCADE ON UPDATE CASCADE,
  amount double precision NOT NULL,
  annotation character varying(255),
  inserted_at timestamp without time zone NOT NULL,
  updated_at timestamp without time zone NOT NULL,
  unit_id bigint REFERENCES ingredient_units (id) ON DELETE CASCADE ON UPDATE CASCADE
);

