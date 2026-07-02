pub mod dao;
pub mod employee;
pub mod payroll;
pub mod auth;
pub mod health;

pub use health::health_check;
pub use dao::{create_dao, get_dao};
pub use employee::{add_employee, get_employees, update_employee, remove_employee};
pub use payroll::{
    create_payroll, approve_payroll, execute_payroll,
    claim_payroll, get_payrolls,
    deposit_to_treasury, get_treasury_balance,
    create_proposal, approve_proposal, get_proposals,
};
pub use auth::{login, register, refresh_token};
