use crate::tree_lib::{Deepest, VertexKey, Vertices};

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

impl Serialize for Vertices {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(&k.to_string(), &v)?;
        }
        map.end()
    }
}

struct VerticesDeserializer;

impl<'de> Visitor<'de> for VerticesDeserializer {
    type Value = Vertices;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("Vertices key value map.")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut new_obj = Vertices::default();

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = map.next_entry()? {
            // explictly delare type for key
            let key_string: String = key;

            new_obj
                .0
                .insert(key_string.parse::<u32>().unwrap(), Arc::new(value));
        }

        Ok(new_obj)
    }
}

impl<'de> Deserialize<'de> for Vertices {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(VerticesDeserializer)
    }
}

impl Serialize for Deepest {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for element in &self.0 {
            seq.serialize_element(&element.to_string())?;
        }
        seq.end()
    }
}

struct DeepestDeserializer;

impl<'de> Visitor<'de> for DeepestDeserializer {
    type Value = Deepest;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("Deepest order set seq.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut new_obj = Deepest::default();

        while let Some(key) = seq.next_element()? {
            // explictly delare type for key
            let key_string: String = key;
            // see Display implementation for `VertexKey`
            let split: Vec<&str> = key_string.split("_").collect();
            let vertex_key = VertexKey::new(
                split[0].parse::<u32>().unwrap(),
                split[1].parse::<u32>().unwrap(),
            );

            new_obj.0.insert(vertex_key);
        }

        Ok(new_obj)
    }
}

impl<'de> Deserialize<'de> for Deepest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(DeepestDeserializer)
    }
}

#[cfg(test)]
mod tests {
    use crate::tree_lib::Tree;
    use crate::tree_lib::Vertex;

    #[test]
    fn test_serde_vertex() {
        let mut tree = Tree::default().insert_vertex(0).unwrap();
        for i in 0u32..20 {
            tree = tree.insert_vertex(i).unwrap();
        }

        for i in 0u32..21 {
            let vertex = tree.get_vertex(i).unwrap();
            let serialized_vertex = serde_json::to_string(&vertex).unwrap();
            let parsed_vertex: Vertex =
                serde_json::from_str(&serialized_vertex).unwrap();

            assert_eq!(
                vertex.clone(),
                parsed_vertex,
                "Vertex should match via serde conversion"
            );
        }
    }

    #[test]
    fn test_serde_tree() {
        let mut tree = Tree::default().insert_vertex(0).unwrap();
        for i in 0u32..20 {
            tree = tree.insert_vertex(i).unwrap();
        }

        let serialized_tree = serde_json::to_string(&tree).unwrap();

        println!("serialized_tree: {}", serialized_tree);

        let parsed_tree: Tree = serde_json::from_str(&serialized_tree).unwrap();

        assert_eq!(
            format!("{:?}", tree),
            format!("{:?}", parsed_tree),
            "Tree should match via serde conversion"
        );
    }
}
