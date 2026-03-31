-- Every row is scoped by tenant_id and user_id; the engine sends both on every call.

create table conversations (
    id            uuid        primary key,
    tenant_id     text        not null,
    user_id       text        not null,
    channel_id    text        not null,
    visibility    text        not null check (visibility in ('public', 'private')),
    last_position bigint      not null default 0,
    ended_at      timestamptz,
    created_at    timestamptz not null default now(),
    updated_at    timestamptz not null default now()
);

create index conversations_open_by_channel
    on conversations (tenant_id, user_id, channel_id, updated_at desc)
    where ended_at is null;

create table messages (
    conversation_id uuid        not null references conversations (id) on delete cascade,
    position        bigint      not null,
    role            text        not null check (role in ('system', 'user', 'assistant', 'tool', 'summary')),
    content         jsonb       not null,
    covers          bigint,
    created_at      timestamptz not null default now(),
    primary key (conversation_id, position)
);

create index messages_summaries
    on messages (conversation_id, position desc)
    where role = 'summary';

create table memories (
    id         uuid        primary key,
    tenant_id  text        not null,
    user_id    text        not null,
    kind       text        not null check (kind in ('episodic', 'semantic', 'profile', 'task')),
    content    text        not null,
    confidence real        not null check (confidence between 0 and 1),
    created_at timestamptz not null default now(),
    expires_at timestamptz
);

create index memories_by_user on memories (tenant_id, user_id, created_at desc);

create table profile_nodes (
    id         uuid        primary key,
    tenant_id  text        not null,
    user_id    text        not null,
    kind       text        not null,
    label      text        not null,
    confidence real        not null check (confidence between 0 and 1),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create unique index profile_nodes_key on profile_nodes (tenant_id, user_id, kind, lower(label));

create table profile_relations (
    subject_id uuid        not null references profile_nodes (id) on delete cascade,
    relation   text        not null,
    object_id  uuid        not null references profile_nodes (id) on delete cascade,
    confidence real        not null check (confidence between 0 and 1),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    primary key (subject_id, relation, object_id)
);

create index profile_relations_by_object on profile_relations (object_id);

create table pending_actions (
    token        uuid        primary key,
    tenant_id    text        not null,
    user_id      text        not null,
    action       jsonb       not null,
    payload_hash text        not null,
    created_at   timestamptz not null default now(),
    expires_at   timestamptz not null
);

create index pending_actions_expiry on pending_actions (expires_at);
