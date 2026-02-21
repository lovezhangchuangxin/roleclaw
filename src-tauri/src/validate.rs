use crate::domain::{EventLogEntry, GameEvent, SaveSnapshot, WorldCard};
use std::collections::HashSet;

fn is_valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    if bytes.len() < 3 || bytes.len() > 64 {
        return false;
    }
    let first = bytes[0] as char;
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    bytes.iter().all(|b| {
        let c = *b as char;
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'
    })
}

fn parse_major(schema_version: &str) -> Option<u32> {
    schema_version.split('.').next()?.parse::<u32>().ok()
}

pub fn validate_world_card(card: &WorldCard) -> Result<(), String> {
    if !is_valid_id(&card.id) {
        return Err("worldCard.id 不合法，需匹配 ^[a-z0-9][a-z0-9_-]{2,63}$".to_string());
    }
    if card.name.trim().is_empty() || card.name.chars().count() > 80 {
        return Err("worldCard.name 长度需为 1~80".to_string());
    }
    if card.content_version < 1 {
        return Err("worldCard.contentVersion 必须 >= 1".to_string());
    }

    let major = parse_major(&card.schema_version)
        .ok_or_else(|| "worldCard.schemaVersion 必须是 MAJOR.MINOR.PATCH".to_string())?;
    if major != 2 {
        return Err(format!(
            "worldCard.schemaVersion={} 当前不兼容，仅支持 2.x.x，请使用新版编辑器重建",
            card.schema_version
        ));
    }

    if card.worldbook.title.trim().is_empty() {
        return Err("worldCard.worldbook.title 不能为空".to_string());
    }
    if card.worldbook.overview.trim().is_empty() {
        return Err("worldCard.worldbook.overview 不能为空".to_string());
    }

    if card.map.nodes.is_empty() {
        return Err("worldCard.map.nodes 不能为空".to_string());
    }

    let mut node_ids: HashSet<&str> = HashSet::new();
    for (idx, node) in card.map.nodes.iter().enumerate() {
        if node.id.trim().is_empty() {
            return Err(format!("worldCard.map.nodes[{idx}].id 不能为空"));
        }
        if node.name.trim().is_empty() {
            return Err(format!("worldCard.map.nodes[{idx}].name 不能为空"));
        }
        if node.tags.len() > 32 {
            return Err(format!("worldCard.map.nodes[{idx}].tags 最多 32 项"));
        }
        if !node_ids.insert(node.id.as_str()) {
            return Err(format!("worldCard.map.nodes[{idx}].id 重复: {}", node.id));
        }
    }

    if !node_ids.contains(card.map.start_node_id.as_str()) {
        return Err("worldCard.map.startNodeId 不存在于 map.nodes".to_string());
    }

    let mut undirected_edges: HashSet<String> = HashSet::new();
    for (idx, edge) in card.map.edges.iter().enumerate() {
        if edge.id.trim().is_empty() {
            return Err(format!("worldCard.map.edges[{idx}].id 不能为空"));
        }
        if !node_ids.contains(edge.a.as_str()) || !node_ids.contains(edge.b.as_str()) {
            return Err(format!("worldCard.map.edges[{idx}] 端点不在 nodes 中"));
        }
        if edge.a == edge.b {
            return Err(format!("worldCard.map.edges[{idx}] 不允许自环"));
        }
        let key = if edge.a < edge.b {
            format!("{}::{}", edge.a, edge.b)
        } else {
            format!("{}::{}", edge.b, edge.a)
        };
        if !undirected_edges.insert(key) {
            return Err(format!("worldCard.map.edges[{idx}] 存在重复无向边"));
        }
    }

    for (idx, npc) in card.npcs.iter().enumerate() {
        if npc.id.trim().is_empty() {
            return Err(format!("worldCard.npcs[{idx}].id 不能为空"));
        }
        if npc.name.trim().is_empty() {
            return Err(format!("worldCard.npcs[{idx}].name 不能为空"));
        }
        if npc.identity.trim().is_empty() {
            return Err(format!("worldCard.npcs[{idx}].identity 不能为空"));
        }
    }

    let mut event_ids: HashSet<&str> = HashSet::new();
    for (idx, event) in card.events.iter().enumerate() {
        if event.id.trim().is_empty() {
            return Err(format!("worldCard.events[{idx}].id 不能为空"));
        }
        if event.name.trim().is_empty() {
            return Err(format!("worldCard.events[{idx}].name 不能为空"));
        }
        if event.prompt.trim().is_empty() {
            return Err(format!("worldCard.events[{idx}].prompt 不能为空"));
        }
        if !event_ids.insert(event.id.as_str()) {
            return Err(format!("worldCard.events[{idx}].id 重复: {}", event.id));
        }
    }

    let mut chapter_ids: HashSet<&str> = HashSet::new();
    for (idx, goal) in card.chapter_goals.iter().enumerate() {
        if goal.id.trim().is_empty() {
            return Err(format!("worldCard.chapterGoals[{idx}].id 不能为空"));
        }
        if goal.title.trim().is_empty() {
            return Err(format!("worldCard.chapterGoals[{idx}].title 不能为空"));
        }
        if goal.prompt.trim().is_empty() {
            return Err(format!("worldCard.chapterGoals[{idx}].prompt 不能为空"));
        }
        if !chapter_ids.insert(goal.id.as_str()) {
            return Err(format!("worldCard.chapterGoals[{idx}].id 重复: {}", goal.id));
        }
    }

    Ok(())
}

pub fn validate_save_snapshot(snapshot: &SaveSnapshot) -> Result<(), String> {
    if snapshot.save_id.trim().is_empty() {
        return Err("snapshot.saveId 不能为空".to_string());
    }
    if snapshot.current_location_id.trim().is_empty() {
        return Err("snapshot.currentLocationId 不能为空".to_string());
    }
    Ok(())
}

pub fn validate_event_log_entry(log: &EventLogEntry) -> Result<(), String> {
    if log.timestamp.trim().is_empty() {
        return Err("eventLog.timestamp 不能为空".to_string());
    }
    Ok(())
}

pub fn validate_game_event(event: &GameEvent) -> Result<(), String> {
    if event.id.trim().is_empty() || event.name.trim().is_empty() {
        return Err("gameEvent.id/name 不能为空".to_string());
    }
    if event.actions.is_empty() {
        return Err(format!("gameEvent {} actions 不能为空", event.id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        CardPromptEvent, ChapterGoal, MapCanvas, MapEdge, MapNode, NpcProfile, WorldBook, WorldMap,
    };

    fn valid_event() -> CardPromptEvent {
        CardPromptEvent {
            id: "evt_001".to_string(),
            name: "事件".to_string(),
            prompt: "推动剧情".to_string(),
        }
    }

    fn valid_chapter() -> ChapterGoal {
        ChapterGoal {
            id: "ch_1".to_string(),
            title: "章节一".to_string(),
            prompt: "当前应聚焦调查主线线索".to_string(),
        }
    }

    fn valid_card() -> WorldCard {
        WorldCard {
            id: "card_001".to_string(),
            name: "测试世界".to_string(),
            schema_version: "2.0.0".to_string(),
            content_version: 1,
            worldbook: WorldBook {
                title: "测试世界".to_string(),
                overview: "概述".to_string(),
                background: "背景".to_string(),
                core_conflicts: vec![],
                play_style: "线性".to_string(),
            },
            map: WorldMap {
                nodes: vec![MapNode {
                    id: "loc_001".to_string(),
                    name: "地点".to_string(),
                    description: "desc".to_string(),
                    tags: vec!["tag".to_string()],
                    x: 0.0,
                    y: 0.0,
                }],
                edges: vec![],
                start_node_id: "loc_001".to_string(),
                canvas: MapCanvas {
                    width: 800,
                    height: 600,
                },
            },
            npcs: vec![NpcProfile {
                id: "npc_001".to_string(),
                name: "NPC".to_string(),
                personality: vec!["trait".to_string()],
                identity: "引导者".to_string(),
            }],
            events: vec![valid_event()],
            chapter_goals: vec![valid_chapter()],
        }
    }

    #[test]
    fn validate_world_card_v2_valid_passes() {
        let card = valid_card();
        assert!(validate_world_card(&card).is_ok());
    }

    #[test]
    fn validate_world_card_v2_rejects_old_schema() {
        let mut card = valid_card();
        card.schema_version = "1.2.3".to_string();
        assert!(validate_world_card(&card).is_err());
    }

    #[test]
    fn validate_world_card_v2_rejects_invalid_edge() {
        let mut card = valid_card();
        card.map.edges.push(MapEdge {
            id: "e2".to_string(),
            a: "loc_001".to_string(),
            b: "loc_001".to_string(),
            locked: false,
            unlock_conditions: vec![],
        });
        assert!(validate_world_card(&card).is_err());
    }

    #[test]
    fn validate_world_card_v2_rejects_empty_event_prompt() {
        let mut card = valid_card();
        card.events[0].prompt = "".to_string();
        assert!(validate_world_card(&card).is_err());
    }
}
