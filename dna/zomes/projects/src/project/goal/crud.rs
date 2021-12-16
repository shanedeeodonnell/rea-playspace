use crate::project::{
    edge::crud::{inner_archive_edge, inner_create_edge, inner_fetch_edges, Edge},
    entry_point::crud::{inner_archive_entry_point, inner_fetch_entry_points},
    goal_comment::crud::{inner_archive_goal_comment, inner_fetch_goal_comments},
    goal_member::crud::{archive_goal_members, GoalMember},
    goal_vote::crud::{inner_archive_goal_vote, inner_fetch_goal_votes},
};
use crate::{get_peers_content, SignalType};
use hdk::prelude::*;
use hdk_crud::{
    crud,
    retrieval::retrieval::FetchOptions,
    signals::{ActionSignal, ActionType},
    wire_element::WireElement,
};
use holo_hash::{AgentPubKeyB64, HeaderHashB64};
use std::fmt;

// A Goal Card. This is a card on the SoA Tree which can be small or non-small, complete or
// incomplete, certain or uncertain, and contains text content.
// user hash and unix timestamp are included to prevent hash collisions.
#[hdk_entry(id = "goal")]
#[derive(Clone, PartialEq)]
pub struct Goal {
    pub content: String,
    pub user_hash: AgentPubKeyB64,
    pub user_edit_hash: Option<AgentPubKeyB64>,
    pub timestamp_created: f64,
    pub timestamp_updated: Option<f64>,
    pub hierarchy: Hierarchy,
    pub status: Status,
    pub tags: Option<Vec<String>>,
    pub description: String,
    pub time_frame: Option<TimeFrame>,
    pub is_imported: bool,
}

impl Goal {
    pub fn new(
        content: String,
        user_hash: AgentPubKeyB64,
        user_edit_hash: Option<AgentPubKeyB64>,
        timestamp_created: f64,
        timestamp_updated: Option<f64>,
        hierarchy: Hierarchy,
        status: Status,
        tags: Option<Vec<String>>,
        description: String,
        time_frame: Option<TimeFrame>,
        is_imported: bool,
    ) -> Self {
        Self {
            content,
            user_hash,
            user_edit_hash,
            timestamp_created,
            timestamp_updated,
            hierarchy,
            status,
            tags,
            description,
            time_frame,
            is_imported,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
pub struct UIEnum(pub String);

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
#[serde(from = "UIEnum")]
#[serde(into = "UIEnum")]
pub enum Status {
    Uncertain,
    Incomplete,
    InProcess,
    Complete,
    InReview,
}

impl From<UIEnum> for Status {
    fn from(ui_enum: UIEnum) -> Self {
        match ui_enum.0.as_str() {
            "Incomplete" => Self::Incomplete,
            "InProcess" => Self::InProcess,
            "Complete" => Self::Complete,
            "InReview" => Self::InReview,
            _ => Self::Uncertain,
        }
    }
}
impl From<Status> for UIEnum {
    fn from(status: Status) -> Self {
        Self(status.to_string())
    }
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
#[serde(from = "UIEnum")]
#[serde(into = "UIEnum")]
pub enum Hierarchy {
    Root,
    Trunk,
    Branch,
    Leaf,
    NoHierarchy,
}
impl From<UIEnum> for Hierarchy {
    fn from(ui_enum: UIEnum) -> Self {
        match ui_enum.0.as_str() {
            "Root" => Self::Root,
            "Trunk" => Self::Trunk,
            "Branch" => Self::Branch,
            "Leaf" => Self::Leaf,
            _ => Self::NoHierarchy,
        }
    }
}
impl From<Hierarchy> for UIEnum {
    fn from(hierarchy: Hierarchy) -> Self {
        Self(hierarchy.to_string())
    }
}
impl fmt::Display for Hierarchy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
pub struct TimeFrame {
    from_date: f64,
    to_date: f64,
}

impl TimeFrame {
    pub fn new(from_date: f64, to_date: f64) -> Self {
        Self { from_date, to_date }
    }
}

fn convert_to_receiver_signal(signal: ActionSignal<Goal>) -> SignalType {
    SignalType::Goal(signal)
}

crud!(
    Goal,
    goal,
    "goal",
    get_peers_content,
    convert_to_receiver_signal
);

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
#[serde(from = "UIEnum")]
#[serde(into = "UIEnum")]
pub enum RelationInput {
    ExistingGoalAsChild,
    ExistingGoalAsParent,
}
impl From<UIEnum> for RelationInput {
    fn from(ui_enum: UIEnum) -> Self {
        match ui_enum.0.as_str() {
            "ExistingGoalAsChild" => Self::ExistingGoalAsChild,
            "ExistingGoalAsParent" => Self::ExistingGoalAsParent,
            _ => Self::ExistingGoalAsChild,
        }
    }
}
impl From<RelationInput> for UIEnum {
    fn from(relation_input: RelationInput) -> Self {
        Self(relation_input.to_string())
    }
}
impl fmt::Display for RelationInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
pub struct LinkedGoalDetails {
    goal_address: HeaderHashB64,
    relation: RelationInput,
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
pub struct CreateGoalWithEdgeInput {
    entry: Goal,
    maybe_linked_goal: Option<LinkedGoalDetails>,
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
pub struct CreateGoalWithEdgeOutput {
    goal: WireElement<Goal>,
    maybe_edge: Option<WireElement<Edge>>,
}

// custom signal type
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalWithEdgeSignal {
    entry_type: String,
    action: ActionType,
    data: CreateGoalWithEdgeOutput,
}

#[hdk_extern]
pub fn create_goal_with_edge(
    input: CreateGoalWithEdgeInput,
) -> ExternResult<CreateGoalWithEdgeOutput> {
    // false to say don't send a signal
    let wire_element = inner_create_goal(input.entry.clone(), false, None)?;
    let new_goal_address = wire_element.header_hash.clone();
    let maybe_edge: Option<WireElement<Edge>> = match input.maybe_linked_goal {
        Some(linked_goal_details) => {
            let (parent_address, child_address) = match linked_goal_details.relation {
                // new goal becomes parent
                RelationInput::ExistingGoalAsChild => {
                    (new_goal_address, linked_goal_details.goal_address)
                }
                // new goal becomes child
                RelationInput::ExistingGoalAsParent => {
                    (linked_goal_details.goal_address, new_goal_address)
                }
            };
            let random = sys_time()?;
            let r0 = random.as_millis();
            let edge = Edge {
                parent_address,
                child_address,
                randomizer: r0,
                is_imported: false,
            };
            let edge_wire_element = inner_create_edge(edge, false, None)?;
            Some(edge_wire_element)
        }
        None => None,
    };

    let goal_with_edge = CreateGoalWithEdgeOutput {
        goal: wire_element.clone(),
        maybe_edge,
    };
    let signal = SignalType::GoalWithEdge(GoalWithEdgeSignal {
        entry_type: "goal_with_edge".to_string(),
        action: ActionType::Create,
        data: goal_with_edge.clone(),
    });
    let payload = ExternIO::encode(signal)?;
    let peers = get_peers_content()?;
    remote_signal(payload, peers)?;

    Ok(goal_with_edge)
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
pub struct ArchiveGoalFullyResponse {
    address: HeaderHashB64,
    archived_edges: Vec<HeaderHashB64>,
    archived_goal_members: Vec<HeaderHashB64>,
    archived_goal_votes: Vec<HeaderHashB64>,
    archived_goal_comments: Vec<HeaderHashB64>,
    archived_entry_points: Vec<HeaderHashB64>,
}

// custom signal type
#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct ArchiveGoalFullySignal {
    entry_type: String,
    action: ActionType,
    data: ArchiveGoalFullyResponse,
}

#[hdk_extern]
pub fn archive_goal_fully(address: HeaderHashB64) -> ExternResult<ArchiveGoalFullyResponse> {
    inner_archive_goal(address.clone(), false)?;

    let archived_edges = inner_fetch_edges(FetchOptions::All, GetOptions::content())?
        .into_iter()
        .filter(|wire_element| {
            // check whether the parent_address or child_address is equal to the given address.
            // If so, the edge is connected to the goal being archived.
            wire_element.entry.child_address == address.clone()
                || wire_element.entry.parent_address == address.clone()
        })
        .map(|wire_element| {
            let edge_address = wire_element.header_hash;
            match inner_archive_edge(edge_address.clone(), false) {
                Ok(_) => Ok(edge_address),
                Err(e) => Err(e),
            }
        })
        // filter out errors
        .filter_map(Result::ok)
        .collect();

    let archived_goal_members = archive_goal_members(address.clone())?;

    let archived_goal_votes = inner_fetch_goal_votes(FetchOptions::All, GetOptions::content())?
        .into_iter()
        .filter(|wire_element| wire_element.entry.goal_address == address.clone())
        .map(|wire_element| {
            let goal_vote_address = wire_element.header_hash;
            match inner_archive_goal_vote(goal_vote_address.clone(), false) {
                Ok(_) => Ok(goal_vote_address),
                Err(e) => Err(e),
            }
        })
        // filter out errors
        .filter_map(Result::ok)
        .collect();

    let archived_goal_comments =
        inner_fetch_goal_comments(FetchOptions::All, GetOptions::content())?
            .into_iter()
            .filter(|wire_element| wire_element.entry.goal_address == address)
            .map(|wire_element| {
                let goal_comment_address = wire_element.header_hash;
                match inner_archive_goal_comment(goal_comment_address.clone(), false) {
                    Ok(_) => Ok(goal_comment_address),
                    Err(e) => Err(e),
                }
            })
            // filter out errors
            .filter_map(Result::ok)
            .collect();

    let archived_entry_points = inner_fetch_entry_points(FetchOptions::All, GetOptions::content())?
        .into_iter()
        .filter(|wire_element| wire_element.entry.goal_address == address)
        .map(|wire_element| {
            let entry_point_address = wire_element.header_hash;
            match inner_archive_entry_point(entry_point_address.clone(), false) {
                Ok(_) => Ok(entry_point_address),
                Err(e) => Err(e),
            }
        })
        // filter out errors
        .filter_map(Result::ok)
        .collect();

    let archive_response = ArchiveGoalFullyResponse {
        address,
        archived_edges,
        archived_goal_members,
        archived_goal_votes,
        archived_goal_comments,
        archived_entry_points,
    };

    let signal = SignalType::ArchiveGoalFully(ArchiveGoalFullySignal {
        entry_type: "archive_goal_fully".to_string(),
        action: ActionType::Delete,
        data: archive_response.clone(),
    });
    let payload = ExternIO::encode(signal)?;
    let peers = get_peers_content()?;
    remote_signal(payload, peers)?;

    Ok(archive_response)
}

// TODO: finish get goal history
#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone)]
pub struct GetHistoryResponse {
    entries: Vec<Goal>,
    members: Vec<Vec<GoalMember>>,
    address: HeaderHashB64,
}

// pub fn history_of_goal(address: HeaderHashB64) -> ExternResult<GetHistoryResponse> {
//   let anchor_address = Entry::App(
//     "anchor".into(),       // app entry type
//     "goal_members".into(), // app entry value
//   )
//   .address();
//   // return all the Goal objects from the entries linked to the edge anchor (drop entries with wrong type)
//   let members = get_links!(
//     &anchor_address,
//     LinkMatch::Exactly("anchor->goal_member"), // the link type to match
//     LinkMatch::Any,
//   )?
//   // scoop all these entries up into an array and return it
//   .addresses()
//   .into_iter()
//   .map(|member_address: HeaderHashB64| {
//     if let Ok(Some(entry_history)) = hdk::api::get_entry_history(&member_address) {
//       Some(
//         entry_history
//           .items
//           .into_iter()
//           .map(|item| {
//             if let Some(App(_, value_entry)) = item.entry {
//               match serde_json::from_str::<GoalMember>(&Into::<String>::into(value_entry)).ok() {
//                 Some(goal_member) => {
//                   // filter down to only Goal Members that are associated with the requested Goal
//                   if goal_member.goal_address == address {
//                     Ok(goal_member)
//                   } else {
//                     Err(ZomeApiError::Internal("error".into()))
//                   }
//                 }
//                 None => Err(ZomeApiError::Internal("error".into())),
//               }
//             } else {
//               Err(ZomeApiError::Internal("error".into()))
//             }
//           })
//           .filter_map(Result::ok)
//           .collect(),
//       )
//     } else {
//       None
//     }
//   })
//   .filter_map(|op: Option<Vec<GoalMember>>| match op {
//     Some(vec) => {
//       if vec.len() > 0 {
//         Some(vec)
//       } else {
//         None
//       }
//     }
//     _ => None,
//   })
//   .collect();
//   if let Ok(Some(entry_history)) = hdk::api::get_entry_history(&address) {
//     Ok(GetHistoryResponse {
//       entries: entry_history
//         .items
//         .into_iter()
//         .map(|item| {
//           if let Some(App(_, value_entry)) = item.entry {
//             match serde_json::from_str::<Goal>(&Into::<String>::into(value_entry)).ok() {
//               Some(goal) => Ok(goal),
//               None => Err(ZomeApiError::Internal("error".into())),
//             }
//           } else {
//             Err(ZomeApiError::Internal("error".into()))
//           }
//         })
//         .filter_map(Result::ok)
//         .collect(),
//       members: members,
//       address: address,
//     })
//   } else {
//     Err(ZomeApiError::Internal("error".into()))
//   }
// }
