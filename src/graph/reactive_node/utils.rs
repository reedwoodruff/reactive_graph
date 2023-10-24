use im::{HashMap, HashSet};

use crate::prelude::{EdgeDescriptor, EdgeFinder, GraphTraits};

pub fn search_map_for_edge<E: GraphTraits, I>(
    edge_finder: &EdgeFinder<E>,
    map: &HashMap<E, I>,
) -> HashSet<EdgeDescriptor<E>>
where
    I: IntoIterator<Item = EdgeDescriptor<E>> + Clone,
{
    let mut found_edges = HashSet::new();
    for (edge_type, edges) in map.iter() {
        // Return the array of one if the edgefinder is not set to match all
        if !found_edges.is_empty()
            && (edge_finder.match_all.is_none()
                || (edge_finder.match_all.is_some() && !edge_finder.match_all.unwrap()))
        {
            break;
        }

        if edge_finder.edge_type.as_ref().is_some()
            && !edge_finder.edge_type.as_ref().unwrap().contains(edge_type)
        {
            continue;
        }
        for edge in edges.clone().into_iter() {
            // Return the array of one if the edgefinder is not set to match all
            if !found_edges.is_empty()
                && (edge_finder.match_all.is_none()
                    || (edge_finder.match_all.is_some() && !edge_finder.match_all.unwrap()))
            {
                break;
            }

            if edge_finder.dir.as_ref().is_some() && edge_finder.dir.as_ref().unwrap() != &edge.dir
            {
                panic!("Edge direction does not match");
            }

            if edge_finder.host.is_some()
                && !edge_finder.host.as_ref().unwrap().contains(&edge.host)
            {
                continue;
            }
            if edge_finder.target.is_some()
                && !edge_finder.target.as_ref().unwrap().contains(&edge.target)
            {
                continue;
            }
            if edge_finder.render_info.is_some()
                && edge_finder.render_info.as_ref().unwrap() != &edge.render_info
            {
                continue;
            }
            found_edges.insert(edge.clone());
        }
    }
    found_edges
}
