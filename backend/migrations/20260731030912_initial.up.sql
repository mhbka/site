-- =========================================================
-- Blog schema for Supabase (Postgres)
-- Assumes Supabase's built-in `auth.users` table for identity.
-- Run this in the Supabase SQL editor, or via `supabase db push`.
-- =========================================================

create extension if not exists pgcrypto; -- for gen_random_uuid()

-- ---------------------------------------------------------
-- POSTS
-- ---------------------------------------------------------
create table posts (
  id                 uuid primary key default gen_random_uuid(),
  author_id          uuid not null references auth.users(id) on delete cascade,

  title              text not null,
  slug               text not null unique,
  excerpt            text,

  content_md         text not null default '',
  content_html       text not null default '', -- rendered server-side on save/publish

  status             text not null default 'draft'
                        check (status in ('draft', 'scheduled', 'published')),
  published_at       timestamptz,               -- set on publish; future = scheduled

  seo_description    text,
  og_image_url       text,
  reading_time_min   int,

  preview_token      uuid not null default gen_random_uuid(), -- share unpublished drafts

  created_at         timestamptz not null default now(),
  updated_at         timestamptz not null default now(),
  deleted_at         timestamptz                -- soft delete
);

create index idx_posts_status_published on posts (status, published_at desc)
  where deleted_at is null;
create index idx_posts_author on posts (author_id);

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
-- POST REVISIONS  (insert-only history, cheap now / painful to add later)
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
-- TAGS
-- ---------------------------------------------------------
create table tags (
  id    uuid primary key default gen_random_uuid(),
  name  text not null unique,
  slug  text not null unique
);

create table post_tags (
  post_id  uuid not null references posts(id) on delete cascade,
  tag_id   uuid not null references tags(id) on delete cascade,
  primary key (post_id, tag_id)
);

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
  updated_at         timestamptz not null default now()
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

-- =========================================================
-- ROW LEVEL SECURITY
-- =========================================================
alter table posts enable row level security;
alter table post_revisions enable row level security;
alter table tags enable row level security;
alter table post_tags enable row level security;
alter table comments enable row level security;
alter table media enable row level security;

-- Anyone can read published posts; authors can read all their own (incl. drafts)
create policy posts_select_published on posts
  for select
  using (
    (status = 'published' and published_at <= now() and deleted_at is null)
    or auth.uid() = author_id
  );

-- Only the author can write/update/delete their own posts
create policy posts_author_write on posts
  for insert with check (auth.uid() = author_id);

create policy posts_author_update on posts
  for update using (auth.uid() = author_id);

create policy posts_author_delete on posts
  for delete using (auth.uid() = author_id);

-- Revisions: readable/writable only by the post's author
create policy revisions_author_all on post_revisions
  for all using (
    auth.uid() = (select author_id from posts where posts.id = post_id)
  );

-- Tags: readable by everyone, writable by any authenticated user
-- (tighten this to an admin role later if you want stricter control)
create policy tags_select_all on tags for select using (true);
create policy tags_authenticated_write on tags
  for insert with check (auth.uid() is not null);

create policy post_tags_select_all on post_tags for select using (true);
create policy post_tags_author_write on post_tags
  for all using (
    auth.uid() = (select author_id from posts where posts.id = post_id)
  );

-- Comments: visible comments readable by all; pending/hidden readable by
-- their own author only. Any authenticated user can post a comment.
-- Only the comment's author can edit/delete it.
create policy comments_select on comments
  for select using (
    status = 'visible' or auth.uid() = author_id
  );

create policy comments_insert on comments
  for insert with check (auth.uid() = author_id);

create policy comments_author_update on comments
  for update using (auth.uid() = author_id);

create policy comments_author_delete on comments
  for delete using (auth.uid() = author_id);

-- Media: uploader can manage their own rows; public read (URLs are public anyway)
create policy media_select_all on media for select using (true);
create policy media_uploader_write on media
  for insert with check (auth.uid() = uploader_id);
create policy media_uploader_delete on media
  for delete using (auth.uid() = uploader_id);