-- =========================================================
-- NOTE: Uses Supabase's built-in `auth.users` table for identity.
-- =========================================================

create extension if not exists pgcrypto; -- for gen_random_uuid()

create type post_status as enum ('draft', 'published');

-- ---------------------------------------------------------
-- POSTS
-- ---------------------------------------------------------
create table posts (
  id                 uuid primary key default gen_random_uuid(),
  author_id          uuid not null references auth.users(id) on delete cascade,

  title              text not null,
  slug               text not null unique,

  content_md         text not null default '', -- stored exactly as received

  status             post_status not null default 'draft',
  published_at       timestamptz,               -- set when published

  thumbnail_url      text,
  tags               text[] not null default '{}',

  created_at         timestamptz not null default now(),
  updated_at         timestamptz not null default now(),
  deleted_at         timestamptz                -- soft delete
);

create index idx_posts_status_published on posts (status, published_at desc)
  where deleted_at is null;
create index idx_posts_author on posts (author_id);
create index idx_posts_tags on posts using gin (tags);

-- keep updated_at fresh
create or replace function set_updated_at()
returns trigger as $$
begin
  new.updated_at = now();
  return new;
end;
$$ language plpgsql;

create trigger trg_posts_updated_at
  before update on posts
  for each row execute function set_updated_at();

-- ---------------------------------------------------------
-- POST REVISIONS
-- ---------------------------------------------------------
create table post_revisions (
  id            uuid primary key default gen_random_uuid(),
  post_id       uuid not null references posts(id) on delete cascade,
  title         text not null,
  content_md    text not null,
  created_by    uuid not null references auth.users(id),
  created_at    timestamptz not null default now()
);

create index idx_revisions_post on post_revisions (post_id, created_at desc);

-- ---------------------------------------------------------
-- COMMENTS  (threaded, moderatable)
-- ---------------------------------------------------------
create table comments (
  id                 uuid primary key default gen_random_uuid(),
  post_id            uuid not null references posts(id) on delete cascade,
  author_id          uuid not null references auth.users(id) on delete cascade,
  parent_comment_id  uuid references comments(id) on delete cascade,

  body               text not null,
  status             text not null default 'visible'
                        check (status in ('visible', 'pending', 'hidden')),

  created_at         timestamptz not null default now(),
  updated_at         timestamptz not null default now(),
  deleted_at         timestamptz
);

create index idx_comments_post on comments (post_id, created_at);

create trigger trg_comments_updated_at
  before update on comments
  for each row execute function set_updated_at();

-- ---------------------------------------------------------
-- MEDIA  (metadata for files uploaded to a bucket)
-- ---------------------------------------------------------
create table media (
  id            uuid primary key default gen_random_uuid(),
  uploader_id   uuid not null references auth.users(id) on delete cascade,
  bucket_path   text not null,   -- e.g. 'post-images/2026/07/abc123.png'
  public_url    text not null,
  content_type  text,
  created_at    timestamptz not null default now()
);

-- ---------------------------------------------------------
-- PROFILES (additional metadata for users)
-- ---------------------------------------------------------
create table profiles (
  user_id   uuid primary key references auth.users(id) on delete cascade,
  is_author boolean not null default false
);
