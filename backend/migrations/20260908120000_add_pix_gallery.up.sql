alter table profiles
  add column is_pix boolean not null default false;

create table pix (
  id           uuid primary key,
  uploader_id  uuid not null references auth.users(id) on delete cascade,
  bucket_path  text not null unique,
  public_url   text not null,
  content_type text not null,
  tags         text[] not null default '{}',
  uploaded_at  timestamptz,
  created_at   timestamptz not null default now()
);

create index idx_pix_uploaded_created
  on pix (uploaded_at desc, created_at desc, id desc)
  where uploaded_at is not null;

create index idx_pix_tags on pix using gin (tags);
