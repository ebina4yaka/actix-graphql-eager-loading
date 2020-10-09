use crate::db::models::{Company, CompanyWithUser, Employment, NewUser, Tag, User};
use crate::graphql::schema::Context;
use diesel::prelude::*;
use diesel::result::Error;
use itertools::Itertools;

pub struct UserRepository;
pub struct TagRepository;
pub struct CompanyRepository;

impl UserRepository {
    pub fn all_users(context: &Context) -> Result<Vec<User>, Error> {
        use crate::schema::users;
        users::table
            .load::<User>(&context.pool.get().unwrap())
            .map(|users| users.into_iter().map_into().collect_vec())
    }
    pub fn insert_user(
        context: &Context,
        new_user: NewUser,
        tags: Vec<String>,
        companies: Vec<String>,
    ) -> Result<User, Error> {
        use crate::schema::{companies, employments, tags, users};
        context.pool.get().unwrap().transaction(|| {
            diesel::insert_into(users::table)
                .values(&new_user)
                .get_result::<User>(&context.pool.get().unwrap())
                .and_then(|user| {
                    let values = tags
                        .into_iter()
                        .map(|tag| (tags::user_id.eq(&user.id), tags::name.eq(tag)))
                        .collect_vec();

                    diesel::insert_into(tags::table)
                        .values(&values)
                        .execute(&context.pool.get().unwrap())?;

                    companies
                        .into_iter()
                        .map(|company_name| {
                            let company = companies::table
                                .filter(companies::name.eq(&company_name))
                                .first::<Company>(&context.pool.get().unwrap())
                                .optional()?;

                            let company = match company {
                                Some(x) => x,
                                _ => diesel::insert_into(companies::table)
                                    .values(companies::name.eq(&company_name))
                                    .get_result::<Company>(&context.pool.get().unwrap())?,
                            };

                            diesel::insert_into(employments::table)
                                .values((
                                    employments::user_id.eq(&user.id),
                                    employments::company_id.eq(&company.id),
                                ))
                                .execute(&context.pool.get().unwrap())?;

                            Ok(company)
                        })
                        .collect::<Result<Vec<_>, diesel::result::Error>>()?;
                    Ok(user)
                })
        })
    }
}

impl TagRepository {
    pub fn user_tags(context: &Context, users: &[User]) -> Result<Vec<Tag>, Error> {
        use crate::schema::tags;
        tags::table
            .filter(tags::user_id.eq_any(users.iter().map(|x| x.id).collect_vec()))
            .load::<Tag>(&context.pool.get().unwrap())
    }
}

impl CompanyRepository {
    pub fn user_companies(
        context: &Context,
        users: &[User],
    ) -> Result<Vec<CompanyWithUser>, Error> {
        use crate::schema::{companies, employments, users};
        companies::table
            .inner_join(employments::table.inner_join(users::table))
            .filter(users::id.eq_any(users.iter().map(|x| x.id).collect_vec()))
            .load::<(Company, (Employment, User))>(&context.pool.get().unwrap())
            .map(|data| {
                data.into_iter()
                    .map(|(company, (_, user))| CompanyWithUser {
                        id: company.id,
                        user_id: user.id,
                        name: company.name,
                    })
                    .collect_vec()
            })
    }
}
