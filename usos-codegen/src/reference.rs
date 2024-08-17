use std::fmt::Display;

use serde::Deserialize;

// name|short_name|description|brief_description|ref_url|auth_options|arguments|returns|errors|result_fields|beta|deprecated|admin_access|is_internal
#[derive(Debug, Deserialize)]
pub(crate) struct MethodReference {
    /// name of the method
    pub(crate) name: String,
    /// name without a path
    pub(crate) short_name: String,
    /// HTML-formatted description of what the method does
    pub(crate) description: String,
    /// brief (max 80 characters), single-line, plain-text description of what the method does
    pub(crate) brief_description: String,
    /// URL of a USOSap Reference webpage with method description
    pub(crate) ref_url: String,
    /// describes authentication requirements for this method
    pub(crate) auth_options: AuthRequirements,
    /// list of dictionaries describing method's parameters
    pub(crate) arguments: Vec<Argument>,
    /// HTML-formatted description method's return value
    pub(crate) returns: String,
    /// HTML-formatted description of possible method exceptions
    pub(crate) errors: String,
    ///  list of method's result fields. Any field can belong to either primary or secondary section. This list serves as a concrete specification and an alternative for the "returns" field in the method description
    pub(crate) result_fields: Vec<Field>,
    /// BETA methods may be altered in a backward-incompatible way
    pub(crate) beta: bool,
    /// in case of non-deprecated methods this will be null
    pub(crate) deprecated: Option<Deprecated>,
    /// true if you have administrative access to this method. You need to sign the request with your Consumer Key in order to access this field.
    /// **Consumer key required!!!
    pub(crate) admin_access: Option<bool>,
    /// true if this method is intended to be used only internally, by USOS API itself. This implies that it is in permanent BETA mode, and it can be altered or removed at any time.
    pub(crate) is_internal: bool,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SignatureRequirement {
    Required,
    Optional,
    Ignored,
}

impl Display for SignatureRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            SignatureRequirement::Ignored => "ignored",
            SignatureRequirement::Optional => "optional",
            SignatureRequirement::Required => "required",
        };

        write!(f, "{res}")
    }
}

// consumer|token|administrative_only|ssl_required|scopes
#[derive(Debug, Deserialize)]
pub(crate) struct AuthRequirements {
    pub(crate) consumer: SignatureRequirement,
    pub(crate) token: SignatureRequirement,
    pub(crate) administrative_only: bool,
    pub(crate) ssl_required: bool,
    pub(crate) scopes: Vec<Scope>,
}

// name|is_required|is_deprecated|default_value|description
#[derive(Debug, Deserialize)]
pub(crate) struct Argument {
    pub(crate) name: String,
    pub(crate) is_required: bool,
    pub(crate) is_deprecated: bool,
    /// [`None`] if parameter doesn't have a default value
    pub(crate) default_value: Option<String>,
    pub(crate) description: String,
}

// name|description|is_primary|is_secondary
#[derive(Debug, Deserialize)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) is_primary: bool,
    pub(crate) is_secondary: bool,
}

// deprecated_by|present_until
#[derive(Debug, Deserialize)]
pub(crate) struct Deprecated {
    pub(crate) deprecated_by: Option<String>,
    pub(crate) present_until: Option<String>,
}

#[derive(Deserialize, Debug, Hash, Eq, PartialEq)]
/// /services/apiref/scopes
pub(crate) enum Scope {
    /// Allows access to get administration documents etc.
    #[serde(rename = "adm_documents")]
    AdministrativeDocuments,
    /// Provides access to user's ID cards data, such as chip uid or expiration date
    #[serde(rename = "cards")]
    Cards,
    /// Allows you to change user preferences (via the uprefs module). You may need some other scopes in order to change or view some of the preferences. Also, the access to some important preferences may be restricted in other ways, i.e. only Administrative Consumers may be allowed to change them.
    #[serde(rename = "change_all_preferences")]
    ChangeAllPreferences,
    /// Provides access to details and results of user's course tests.
    #[serde(rename = "crstests")]
    CourseTests,
    /// Provides access to administrative housing operations on user's behalf. For more information, please visit the housing module.
    #[serde(rename = "dorm_admin")]
    Dorms,
    /// Allows editing user's attributes (the same thet the user can edit on his USOSweb profile page).
    #[serde(rename = "edit_user_attrs")]
    ChangeUserAttributes,
    /// Provides access to user's email address.
    #[serde(rename = "email")]
    Email,
    /// Allows access to user's preferences, push notifications, etc.
    #[serde(rename = "events")]
    Events,
    /// Provides access to grades information.
    #[serde(rename = "grades")]
    Grades,
    /// Allows access to read and write exam reports.
    #[serde(rename = "grades_write")]
    GradesWrite,
    /// Provides access to the mailclient module (in the name of your user). Currently only a small set of methods is available for non-administrative consumers, but this set will be growing.
    #[serde(rename = "mailclient")]
    MailClient,
    /// Provides access to user's personal mobile phone number(s).
    #[serde(rename = "mobile_numbers")]
    MobileNumbers,
    /// Enables your application to perform authorized requests on behalf of the user at any time. By default, Access Tokens expire after a short time period to ensure applications only make requests on behalf of users when they are actively using the application. This scope makes Access Tokens long-lived.
    #[serde(rename = "offline_access")]
    OfflineAccess,
    /// Provides access to email addresses of other users (i.e. the ones related to your user).
    #[serde(rename = "other_emails")]
    OtherEmails,
    /// Allows access to your payments.
    #[serde(rename = "payments")]
    Payments,
    /// Provides access to user's personal data, such as PESEL number, date of birth, etc.
    #[serde(rename = "personal")]
    Personal,
    /// Provides read access to user's photo and his/her photo visibility preferences ("who can see my photo?").
    #[serde(rename = "photo")]
    Photo,
    /// Provides access to results of user's placement tests in foreign languages.
    #[serde(rename = "placement_tests")]
    PlacementTests,
    /// Allows access to official permissions related to the user's session debugging rights. Allows you to get the answer to the question "Is my user permitted to debug the session of user X?". See "can_i_debug" field of the services/users/user method for more information.
    #[serde(rename = "session_debugging_perms")]
    SessionDebugging, // dev only
    /// Provides access to most of the actions within the Clearance Slips module. With this scope you can view, create and edit slips, answer questions and perform any non-administrative action which the user can perform via USOSweb. You will need an additional 'slips_admin' scope if you want to manage slip templates too.
    #[serde(rename = "slips")]
    ClearanceSlips,
    /// Provides access to template management of the "slips" module. That is, it allows you to create and edit questions, mark templates as obsolete etc.
    #[serde(rename = "slips_admin")]
    ClearanceSlipsAdmin,
    /// If your user is a staff member, then this scope provides access to some common student-related data usually visible only to staff members, e.g. student numbers, or broader lists of students' study programmes.
    #[serde(rename = "staff_perspective")]
    StaffPerspective,
    /// Provides access to lists of student's exams, information on their examiners, places the exams take place etc.
    #[serde(rename = "student_exams")]
    StudentExams,
    /// Allows to register and unregister the student from his exams.
    #[serde(rename = "student_exams_write")]
    StudentExamsEdit,
    /// Provides access to lists of programmes, courses, classes and groups which the user attends (as a student).
    #[serde(rename = "studies")]
    Studies,
    /// Allows access to surveys from students point of view.  With this scope you can fetch and fill out surveys.
    #[serde(rename = "surveys_filling")]
    SurveysFilling,
    /// Allows access to reports on surveys that concern user as a lecturer.
    #[serde(rename = "surveys_reports")]
    SurveysReports,
    /// Allows access to editing diploma exam protocols, e.g. signing protocols.
    #[serde(rename = "theses_protocols_write")]
    ThesesProtocolsEdit,
}
