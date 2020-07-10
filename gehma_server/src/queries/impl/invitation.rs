use crate::queries::*;
use crate::Pool;
use chrono::{DateTime, Local};
use core::errors::{InternalServerError, ServiceError};
use core::models::dao::*;
use core::models::dto::*;
use core::models::PhoneNumber;
use diesel::{prelude::*, PgConnection};
use log::{debug, error, info, trace};
use uuid::Uuid;

type IResult<V> = Result<V, ServiceError>;

#[derive(Clone)]
pub struct PgInvitationDao {
    pub pool: Pool,
}

impl PersistentInvitation for PgInvitationDao {
    fn get_all(&self, user: &UserDao) -> IResult<Vec<(InvitationDao, InvitationMemberDao)>> {
        info!("fn get_all()");

        use core::schema::invitation::dsl::*;
        use core::schema::invitation_members::dsl::*;

        let conn: &PgConnection = &self.pool.get().unwrap();

        let member_of = invitation_members
            .filter(user_id.eq(user.id))
            .load::<InvitationMemberDao>(conn)?;

        let mut v = Vec::new();

        for m in member_of.into_iter() {
            let binv = invitation
                .filter(id.eq(m.inv_id))
                .load::<InvitationDao>(conn)
                .map_err(|_db_err| {
                    error!("Invitation {}", _db_err);
                    ServiceError::BadRequest("Database fetch failed".to_string())
                })?;

            let inv = binv
                .first()
                .ok_or_else(|| ServiceError::BadRequest("No invitation found".into()))?;

            v.push((inv.clone(), m));
        }

        //TODO sort

        Ok(v)
    }

    fn get(&self, user: &UserDao, inv_id: i32) -> IResult<(InvitationDao, InvitationMemberDao)> {
        info!("fn get()");

        use core::schema::invitation::dsl::*;
        use core::schema::invitation_members::dsl::*;

        let conn: &PgConnection = &self.pool.get().unwrap();

        let bmember_of = invitation_members
            .filter(user_id.eq(user.id))
            .load::<InvitationMemberDao>(conn)?;

        let member_of = bmember_of
            .first()
            .ok_or_else(|| ServiceError::BadRequest("No invitation (member) found".into()))?;

        let binv = invitation
            .filter(id.eq(member_of.inv_id))
            .load::<InvitationDao>(conn)
            .map_err(|_db_err| {
                error!("Invitation {}", _db_err);
                ServiceError::BadRequest("Database fetch failed".to_string())
            })?;

        let inv = binv
            .first()
            .ok_or_else(|| ServiceError::BadRequest("No invitation found".into()))?;

        Ok((inv.clone(), member_of.clone()))
    }

    fn create_invitation(
        &self,
        user: &UserDao,
        contacts: &[ContactDao],
        data: RequestInvitationCreateDto,
    ) -> IResult<(InvitationDao, InvitationMemberDao)> {
        info!("fn create_invitation()");

        use core::schema::invitation::dsl::*;
        use core::schema::invitation_members::dsl::*;

        let conn: &PgConnection = &self.pool.get().unwrap();

        let inv = InsertInvitationDao {
            originator_user_id: user.id,
            edit_text: data.text.clone(),
            edit_time: data.time.clone(),
            original_time: data.time,
            original_text: data.text,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        };

        let inserted_inv = diesel::insert_into(invitation)
            .values(&inv)
            .get_result::<InvitationDao>(conn)
            .map_err(|_db_error| {
                error!("{:?}", _db_error);
                ServiceError::BadRequest("Cannot insert into invitation".into())
            })?;

        let mem = InvitationMemberDao {
            inv_id: inserted_inv.id,
            user_id: user.id.clone(),
            is_seen: false,
            state: 0,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        };

        let inserted_mem = diesel::insert_into(invitation_members)
            .values(&mem)
            .get_result::<InvitationMemberDao>(conn)
            .map_err(|_db_error| {
                error!("{:?}", _db_error);
                ServiceError::BadRequest("Cannot insert into invitation_members".into())
            })?;

        Ok((inserted_inv, inserted_mem))
    }

    fn update_invitation(
        &self,
        user: &UserDao,
        inv_id: i32,
        data: UpdateInvitationStateDto,
    ) -> IResult<()> {
        info!("fn update_invitation()");

        //use core::schema::invitation::dsl::*;
        use core::schema::invitation_members::dsl::*;

        let conn: &PgConnection = &self.pool.get().unwrap();

        let target = invitation_members.filter(inv_id.eq(inv_id).and(user_id.eq(user.id)));

        let new_state = match data.accept {
            true => 0,
            false => 1,
        };

        diesel::update(target)
            .set((
                updated_at.eq(chrono::Local::now().naive_local()),
                state.eq(new_state),
            ))
            .execute(conn)
            .map_err(|_db_error| {
                error!("db_error: {}", _db_error);
                ServiceError::InternalServerError(InternalServerError::DatabaseError(
                    _db_error.to_string(),
                ))
            })?;

        Ok(())
    }

    fn add_members_to_invitation(
        &self,
        user: &UserDao,
        my_inv_id: i32,
        contacts: &[ContactDao],
    ) -> IResult<(InvitationDao, InvitationMemberDao)> {
        use core::schema::invitation::dsl::{invitation, id};
        use core::schema::invitation_members::dsl::{invitation_members,inv_id, user_id};

        let conn: &PgConnection = &self.pool.get().unwrap();

        let binv = invitation
            .filter(id.eq(my_inv_id))
            .load::<InvitationDao>(conn)
            .map_err(|_db_err| {
                error!("Invitation {}", _db_err);
                ServiceError::BadRequest("Database fetch failed".to_string())
            })?;

        let inv = binv
            .first()
            .ok_or_else(|| ServiceError::BadRequest("No invitation found".into()))?;

        let mem = InvitationMemberDao {
            inv_id: inv.id,
            user_id: user.id.clone(),
            is_seen: false,
            state: 0,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        };

        let inserted_mem = diesel::insert_into(invitation_members)
            .values(&mem)
            .get_result::<InvitationMemberDao>(conn)
            .map_err(|_db_error| {
                error!("{:?}", _db_error);
                ServiceError::BadRequest("Cannot insert into invitation_members".into())
            })?;

        Ok((inv.clone(), inserted_mem.clone()))
    }
}
