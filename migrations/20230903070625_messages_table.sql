CREATE type message_status AS enum (
  'pending',
  'processing',
  'completed',
  'failed'
);

CREATE TABLE messages (
  id serial primary key,
  status message_status not null,
  payload jsonb,
  created_at timestamp not null default now(),
  updated_at timestamp not null default now()  
);