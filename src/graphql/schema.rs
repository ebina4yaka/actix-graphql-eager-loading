use crate::db::repositories;
use juniper::{Executor, FieldResult};
use juniper_eager_loading::{prelude::*, EagerLoading, HasMany};
use juniper_from_schema::graphql_schema_from_file;

use crate::db::manager::DataPgPool;
use crate::db::models;

graphql_schema_from_file!("src/graphql/schema.graphql");

pub struct Context {
    pub pool: DataPgPool,
}
impl juniper::Context for Context {}

pub struct Query;
pub struct Mutation;

impl QueryFields for Query {
    fn field_users(
        &self,
        executor: &Executor<'_, Context>,
        trail: &QueryTrail<'_, User, Walked>,
    ) -> FieldResult<Vec<User>> {
        let model_users = repositories::UserRepository::all_users(executor.context())?;
        let mut users = User::from_db_models(&model_users);
        User::eager_load_all_children_for_each(
            &mut users,
            &model_users,
            executor.context(),
            trail,
        )?;
        Ok(users)
    }
}

impl MutationFields for Mutation {
    fn field_create_user(
        &self,
        executor: &Executor<'_, Context>,
        trail: &QueryTrail<'_, User, Walked>,
        name: String,
        tags: Vec<String>,
        companies: Vec<String>,
    ) -> FieldResult<User> {
        let new_user = models::NewUser { name };
        let model_user = repositories::UserRepository::insert_user(
            executor.context(),
            new_user,
            tags,
            companies,
        )?;
        let user = User::new_from_model(&model_user);
        User::eager_load_all_children(user, &[model_user], &executor.context(), trail)
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone, PartialEq, EagerLoading)]
#[eager_loading(context = Context, error = diesel::result::Error)]
pub struct User {
    user: models::User,

    #[has_many(root_model_field = tag)]
    tags: HasMany<Tag>,

    #[has_many(root_model_field = company)]
    companies: HasMany<CompanyWithUser>,
}

impl UserFields for User {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<juniper::ID> {
        Ok(juniper::ID::new(self.user.id.to_string()))
    }

    fn field_name(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.user.name)
    }

    fn field_tags(
        &self,
        _: &Executor<'_, Context>,
        _: &QueryTrail<'_, Tag, Walked>,
    ) -> FieldResult<&Vec<Tag>> {
        self.tags.try_unwrap().map_err(Into::into)
    }

    fn field_companies(
        &self,
        _: &Executor<'_, Context>,
        _: &QueryTrail<'_, CompanyWithUser, Walked>,
    ) -> FieldResult<&Vec<CompanyWithUser>> {
        self.companies.try_unwrap().map_err(Into::into)
    }
}

#[derive(Debug, Clone, PartialEq, EagerLoading)]
#[eager_loading(context = Context, error = diesel::result::Error)]
pub struct Tag {
    tag: models::Tag,
}

impl TagFields for Tag {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<juniper::ID> {
        Ok(juniper::ID::new(self.tag.id.to_string()))
    }

    fn field_user_id(&self, _: &Executor<'_, Context>) -> FieldResult<juniper::ID> {
        Ok(juniper::ID::new(self.tag.user_id.to_string()))
    }

    fn field_name(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.tag.name)
    }
}

#[derive(Debug, Clone, PartialEq, EagerLoading)]
#[eager_loading(context = Context, error = diesel::result::Error)]
pub struct CompanyWithUser {
    company: models::CompanyWithUser,
}

impl CompanyWithUserFields for CompanyWithUser {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<juniper::ID> {
        Ok(juniper::ID::new(self.company.id.to_string()))
    }

    fn field_name(&self, _: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.company.name)
    }
}

impl juniper_eager_loading::LoadFrom<models::User> for models::Tag {
    type Error = diesel::result::Error;
    type Context = Context;

    fn load(
        users: &[models::User],
        _field_args: &(),
        context: &Self::Context,
    ) -> Result<Vec<models::Tag>, Self::Error> {
        repositories::TagRepository::user_tags(context, users)
    }
}

impl juniper_eager_loading::LoadFrom<models::User> for models::CompanyWithUser {
    type Error = diesel::result::Error;
    type Context = Context;

    fn load(
        users: &[models::User],
        _field_args: &(),
        context: &Self::Context,
    ) -> Result<Vec<models::CompanyWithUser>, Self::Error> {
        repositories::CompanyRepository::user_companies(context, users)
    }
}
