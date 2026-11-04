-- Multiple management IPs (first = primary) and OS family for the tree selector.

ALTER TABLE devices ADD COLUMN mgmt_ips TEXT NOT NULL DEFAULT '[]';

UPDATE devices
SET mgmt_ips = json_array(trim(mgmt_ip))
WHERE mgmt_ip IS NOT NULL AND trim(mgmt_ip) <> '';

ALTER TABLE devices DROP COLUMN mgmt_ip;

ALTER TABLE devices ADD COLUMN os_family TEXT;

UPDATE devices
SET os_family = CASE
    WHEN lower(os) LIKE '%windows%' THEN 'windows'
    WHEN lower(os) LIKE '%linux%'
      OR lower(os) LIKE '%ubuntu%'
      OR lower(os) LIKE '%debian%'
      OR lower(os) LIKE '%centos%'
      OR lower(os) LIKE '%rhel%'
      OR lower(os) LIKE '%red hat%'
      OR lower(os) LIKE '%rocky%'
      OR lower(os) LIKE '%alma%'
      OR lower(os) LIKE '%suse%'
      OR lower(os) LIKE '%arch%'
      OR lower(os) LIKE '%fedora%'
      THEN 'linux'
    ELSE 'other'
  END
WHERE os IS NOT NULL AND trim(os) <> '';
