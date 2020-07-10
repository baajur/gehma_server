use chrono::{DateTime, Local};
use core::errors::ServiceError;
use core::models::dto::*;
use core::models::dao::*;
use core::models::PhoneNumber;
use uuid::Uuid;
use mockall::*;

type IResult<K> = Result<K, ServiceError>;

#[automock]
pub trait PersistentInvitation {
    fn get_all(&self, user: &UserDao) -> IResult<Vec<(InvitationDao, InvitationMemberDao)>>;

    fn get(&self, user: &UserDao, inv_id: i32) -> IResult<(InvitationDao, InvitationMemberDao)>;

    fn create_invitation(&self, user: &UserDao, contacts: &[ContactDao], data: RequestInvitationCreateDto) -> IResult<(InvitationDao, InvitationMemberDao)>;

    fn update_invitation(&self, user: &UserDao, inv_id: i32, data: UpdateInvitationStateDto) -> IResult<()>;

    fn add_members_to_invitation(&self, user: &UserDao, inv_id: i32, contacts: &[ContactDao]) -> IResult<(InvitationDao, InvitationMemberDao)>;
}
