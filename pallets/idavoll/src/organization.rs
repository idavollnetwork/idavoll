/// Organization represents an organization
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub avatar_url: Option<String>,
    pub description: Option<String>,
    pub full_name: Option<String>,
    pub id: Option<i64>,
    pub location: Option<String>,
    pub repo_admin_change_team_access: Option<bool>,
    pub username: Option<String>,
    pub visibility: Option<String>,
    pub website: Option<String>,
}

impl Organization {
    /// Create a builder for this object.
    #[inline]
    pub fn builder() -> OrganizationBuilder {
        OrganizationBuilder {
            body: Default::default(),
        }
    }

    #[inline]
    pub fn admin_get_all_orgs() -> OrganizationGetBuilder {
        OrganizationGetBuilder {
            param_page: None,
            param_limit: None,
        }
    }

    #[inline]
    pub fn org_get_all() -> OrganizationGetBuilder1 {
        OrganizationGetBuilder1 {
            param_page: None,
            param_limit: None,
        }
    }

    #[inline]
    pub fn org_get() -> OrganizationGetBuilder2<crate::generics::MissingOrg> {
        OrganizationGetBuilder2 {
            inner: Default::default(),
            _param_org: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn org_list_current_user_orgs() -> OrganizationGetBuilder3 {
        OrganizationGetBuilder3 {
            param_page: None,
            param_limit: None,
        }
    }

    #[inline]
    pub fn org_list_user_orgs() -> OrganizationGetBuilder4<crate::generics::MissingUsername> {
        OrganizationGetBuilder4 {
            inner: Default::default(),
            _param_username: core::marker::PhantomData,
        }
    }
}
