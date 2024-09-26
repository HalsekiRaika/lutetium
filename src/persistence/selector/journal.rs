use std::fmt::{Display, Formatter};
use crate::persistence::identifier::SequenceId;
use crate::persistence::selector::error::SelectionError;

#[derive(Debug, Clone)]
pub struct SelectionCriteria {
    min: SequenceId,
    max: SequenceId,
}

impl SelectionCriteria {
    pub fn new(min: impl Into<i64>, max: impl Into<i64>) -> Result<SelectionCriteria, SelectionError> {
        let min = min.into();
        let max = max.into();
        if min > max || min >= max {
            return Err(SelectionError)
        }
        Ok(Self { min: SequenceId::new(min), max: SequenceId::new(max) })
    }
    
    pub fn latest() -> SelectionCriteria {
        Self { min: SequenceId::min(), max: SequenceId::max() }
    }
    
    pub fn matches(&self, seq: &SequenceId) -> bool {
        &self.min <= seq && seq <= &self.max
    }
}

impl Display for SelectionCriteria {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("select")
            .field("min", &self.min)
            .field("max", &self.max)
            .finish()
    }
}


#[cfg(test)]
mod test {
    use crate::persistence::identifier::SequenceId;
    use crate::persistence::SelectionCriteria;
    
    #[test]
    fn select() {
        let criteria = SelectionCriteria::new(20, 60).unwrap();
        let mut selection = Vec::new();
        for i in 0..100 {
            let id = SequenceId::new(i);
            if criteria.matches(&id) {
                selection.push(i);
            }
        }
        
        assert_eq!(selection, (20..=60).collect::<Vec<i64>>());
    }
}