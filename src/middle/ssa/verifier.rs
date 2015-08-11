//! Implements a pass that goes over the ssa and checks if the ssa is still valid.
//!
//! This is only for verification and to catch potential mistakes.
#![allow(unused_imports, unused_variables)]
use petgraph::EdgeDirection;
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};

use super::cfg_traits::{CFG, CFGMod};
use super::ssa_traits::SSA;
use super::ssastorage::{NodeData,EdgeData};
use super::ssastorage::SSAStorage;

pub trait Verify: SSA {
	fn verify_block(&self, i: &Self::ActionRef);
	fn verify_expr(&self, i: &Self::ValueRef);
}

impl Verify for SSAStorage {
	fn verify_block(&self, block: &NodeIndex) {
		let edge_count = self.edge_count();
		let node_count = self.node_count();
		// Make sure that we have a valid node first.
		assert!(block.index() < node_count);
		let edges = self.edges_of(block);
		// Every BB can have a maximum of 2 Outgoing CFG Edges.
		assert!(edges.len() < 3);
		for edge in edges.iter() {
			assert!(edge.index() < edge_count);
			match self.g[*edge] {
				EdgeData::Control(i) if i < 2 => {
					// Things to lookout for:
					//  * There must be a minimum of two edges.
					//  * There _must_ be a selector.
					//  * The jump targets need to be valid basic blocks.
					//  * The jump targets must not be the same block.
					assert!(edges.len() == 2);
					assert!(self.selector_of(block).is_some());
					let other_edge = match i {
						0 => self.true_edge_of(block),
						1 => self.false_edge_of(block),
						_ => unreachable!(),
					};
					let target_1 = self.target_of(edge);
					let target_2 = self.target_of(&other_edge);
					assert!(target_1.index() < node_count);
					assert!(target_2.index() < node_count);
					let bb1 = &self.g[target_1];
					let bb2 = &self.g[target_2];
					let valid_block1 = if let NodeData::BasicBlock(_)  = *bb1 {
						true
					} else {
						false
					};
					let valid_block2 = if let NodeData::BasicBlock(_)  = *bb2 {
						true
					} else {
						false
					};
					assert!(valid_block1 && valid_block2);
					assert!(target_1 != target_2);
					// No need to test the next edge.
					break;
				},
				EdgeData::Control(2) => {
					// Things to lookout for:
					//  * There can be only one Unconditional Edge.
					//  * There can be no selector.
					//  * The target block exists and is a valid block.
					//  * Make sure we have not introduced an unconditional jump
					//    which self-loops.
					let target_block = self.target_of(edge);
					assert!(edges.len() == 1);
					assert!(self.selector_of(block).is_none());
					assert!(target_block.index() < node_count);
					let valid_block = if let NodeData::BasicBlock(_) = self.g[target_block] {
						true
					} else {
						false
					};
					assert!(valid_block);
					assert!(*block != target_block);
				},
				_ => panic!("Found something other than a control edge!"),
			}
		}
		// Make sure that this block is reachable.
		let incoming = self.incoming_edges(block);
		assert!((incoming.len() > 0) || *block == self.start_node());
	}

	fn verify_expr(&self, i: &NodeIndex) {
		let node_count = self.node_count();
		// Make sure we have a valid node first.
		assert!(i.index() < node_count);
		let node_data = &self.g[*i];
		match *node_data {
			NodeData::Op(opcode, v) => {
			},
			_ => panic!("Found something other than an expression!"),
		}
	}
}

pub fn verify<T>(ssa: &T) where T: Verify {
	let blocks = ssa.blocks();
	for block in blocks.iter() {
		// assert the qualities of the block first.
		ssa.verify_block(block);
		// Iterate through each node in the block and assert their properties.
		let exprs = ssa.exprs_in(block);
		for expr in exprs.iter() {
			ssa.verify_expr(expr);
		}
	}
}
