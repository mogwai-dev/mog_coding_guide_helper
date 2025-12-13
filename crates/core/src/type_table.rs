//! 型テーブル - typedef名と型情報を管理
//!
//! このモジュールはtypedefで定義された型名と実際の型情報を管理し、
//! パーサーが識別子と型名を区別できるようにする。

use std::collections::HashMap;
use crate::type_system::Type;

/// typedef名と型情報を管理する型テーブル
/// 
/// スコープスタック構造を使用し、内側のスコープから外側のスコープへと
/// 型名を検索できる。最初の要素がグローバルスコープ。
#[derive(Debug, Clone)]
pub struct TypeTable {
    /// スコープのスタック（最初の要素がグローバルスコープ）
    /// 各スコープはtypedef名 -> 実際の型情報のマップ
    scopes: Vec<HashMap<String, Type>>,
}

impl TypeTable {
    /// 新しい型テーブルを作成（グローバルスコープを持つ）
    pub fn new() -> Self {
        TypeTable {
            scopes: vec![HashMap::new()], // グローバルスコープ
        }
    }

    /// 新しいスコープを開始
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// 現在のスコープを終了
    /// 
    /// # Panics
    /// グローバルスコープをpopしようとするとパニックする
    pub fn pop_scope(&mut self) {
        if self.scopes.len() <= 1 {
            panic!("Cannot pop global scope");
        }
        self.scopes.pop();
    }

    /// 現在のスコープのネストレベルを取得（0 = グローバル）
    pub fn scope_depth(&self) -> usize {
        self.scopes.len() - 1
    }

    /// typedef名と型情報を現在のスコープに登録
    pub fn register_type(&mut self, type_name: String, type_info: Type) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(type_name, type_info);
        }
    }

    /// 名前が型名（typedef）かどうかを確認
    /// 最内スコープから外側のスコープへと検索
    pub fn is_type_name(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains_key(name))
    }
    
    /// typedef名から実際の型情報を取得
    /// 最内スコープから外側のスコープへと検索
    pub fn get_type_info(&self, name: &str) -> Option<&Type> {
        self.scopes.iter().rev()
            .find_map(|scope| scope.get(name))
    }

    /// 型名を現在のスコープから削除
    pub fn remove_type(&mut self, name: &str) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.remove(name);
        }
    }

    /// すべてのスコープの型名をクリアし、グローバルスコープのみに戻す
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.scopes.push(HashMap::new());
    }

    /// 全スコープを含めた登録されている型名の総数を取得
    pub fn len(&self) -> usize {
        self.scopes.iter().map(|scope| scope.len()).sum()
    }

    /// 型テーブルが空かどうかを確認（全スコープ）
    pub fn is_empty(&self) -> bool {
        self.scopes.iter().all(|scope| scope.is_empty())
    }

    /// すべてのスコープから型名を取得（重複を除く）
    pub fn get_all_types(&self) -> Vec<String> {
        let mut types = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        // 内側から外側へ検索し、最初に見つかった型名を採用
        for scope in self.scopes.iter().rev() {
            for name in scope.keys() {
                if seen.insert(name.clone()) {
                    types.push(name.clone());
                }
            }
        }
        types
    }
}

impl Default for TypeTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_system::BaseType;
    use crate::span::Span;

    fn dummy_span() -> Span {
        Span {
            start_line: 0,
            start_column: 0,
            end_line: 0,
            end_column: 0,
            byte_start_idx: 0,
            byte_end_idx: 0,
        }
    }

    #[test]
    fn test_register_and_check_type() {
        let mut table = TypeTable::new();
        
        assert!(!table.is_type_name("MyInt"));
        
        let int_type = Type::new(BaseType::Int, dummy_span());
        table.register_type("MyInt".to_string(), int_type);
        assert!(table.is_type_name("MyInt"));
        assert!(!table.is_type_name("OtherType"));
    }

    #[test]
    fn test_get_type_info() {
        let mut table = TypeTable::new();
        
        let int_type = Type::new(BaseType::Int, dummy_span());
        table.register_type("MyInt".to_string(), int_type);
        
        let retrieved = table.get_type_info("MyInt");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().base_type, BaseType::Int);
    }

    #[test]
    fn test_multiple_types() {
        let mut table = TypeTable::new();
        
        table.register_type("size_t".to_string(), Type::new(BaseType::Unsigned, dummy_span()));
        table.register_type("int32_t".to_string(), Type::new(BaseType::Int, dummy_span()));
        table.register_type("MyStruct".to_string(), Type::new(BaseType::Int, dummy_span()));
        
        assert_eq!(table.len(), 3);
        assert!(table.is_type_name("size_t"));
        assert!(table.is_type_name("int32_t"));
        assert!(table.is_type_name("MyStruct"));
        assert!(!table.is_type_name("int"));
    }

    #[test]
    fn test_remove_type() {
        let mut table = TypeTable::new();
        
        table.register_type("TempType".to_string(), Type::new(BaseType::Int, dummy_span()));
        assert!(table.is_type_name("TempType"));
        
        table.remove_type("TempType");
        assert!(!table.is_type_name("TempType"));
    }

    #[test]
    fn test_clear() {
        let mut table = TypeTable::new();
        
        table.register_type("Type1".to_string(), Type::new(BaseType::Int, dummy_span()));
        table.register_type("Type2".to_string(), Type::new(BaseType::Float, dummy_span()));
        assert_eq!(table.len(), 2);
        
        table.clear();
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_get_all_types() {
        let mut table = TypeTable::new();
        
        table.register_type("size_t".to_string(), Type::new(BaseType::Unsigned, dummy_span()));
        table.register_type("FILE".to_string(), Type::new(BaseType::Int, dummy_span()));
        
        let types = table.get_all_types();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"size_t".to_string()));
        assert!(types.contains(&"FILE".to_string()));
    }
}
