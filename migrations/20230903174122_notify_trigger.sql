CREATE FUNCTION notify_trigger() RETURNS trigger AS $trigger$
BEGIN
  PERFORM pg_notify('queue_notifications', 'new message');
  RETURN NEW;
END;
$trigger$ LANGUAGE plpgsql;


CREATE TRIGGER message_trigger BEFORE INSERT ON messages
FOR EACH ROW EXECUTE FUNCTION notify_trigger();
