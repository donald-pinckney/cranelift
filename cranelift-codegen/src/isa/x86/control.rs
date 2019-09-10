use super::registers::{FPR, GPR, RU};
// use crate::bitset::BitSet;
use crate::cursor::{Cursor, FuncCursor};
use crate::flowgraph::ControlFlowGraph;
// use crate::ir::condcodes::{FloatCC, IntCC};
use crate::ir;
use crate::ir::{Function, Inst, InstBuilder};
// use crate::isa::constraints::*;
// use crate::isa::enc_tables::*;
// use crate::isa::encoding::base_size;
// use crate::isa::encoding::RecipeSizing;
// use crate::isa::RegUnit;
// use crate::isa;
use crate::isa::TargetIsa;
use crate::ir::types::*;
// use crate::predicates;
// use crate::regalloc::RegDiversions;


pub fn expand_control_x86(
    inst: ir::Inst,
    func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
) { 
    // Unpack the instruction.
    let (func_ref, old_args) = match func.dfg[inst] {
        ir::InstructionData::Control {
            opcode,
            ref args,
            func_ref,
        } => {
            debug_assert_eq!(opcode, ir::Opcode::Control);
            // assert!(args.len(&func.dfg.value_lists) == 1);
            // TODO: also need to verify the type of the arg
            (func_ref, args.clone())
        }
        _ => panic!("Wanted control: {}", func.dfg.display_inst(inst, None)),
    };

    let ptr_ty = isa.pointer_type();
    let sig = func.dfg.ext_funcs[func_ref].signature;

    println!("func_ref: {:?}, args: {:?}", func_ref, old_args.len(&func.dfg.value_lists));


    // let header_pos = FuncCursor::new(func).at_inst(inst);

    // --------------- Code generation ------------------

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // get the function address
    let callee = pos.ins().func_addr(ptr_ty, func_ref);

    // Get the current continuation id
    let k_addr = pos.ins().global_value(I64, ir::GlobalValue::with_number(0).unwrap());
    let k = pos.ins().load(I32, ir::MemFlags::trusted(), k_addr, 0);

    // Save the current context into index k in the table
    let cont_table_addr = pos.ins().global_value(I64, ir::GlobalValue::with_number(1).unwrap());
    let entry_offset = pos.ins().imul_imm(k, 10 * 8); // TODO: 10 * 8 should be the size in bytes of each context entry.
    let entry_addr = pos.ins().iadd(cont_table_addr, entry_offset);

    
    // let rax_val = pos.ins().copy_to_ssa(I64, RU::rax);

    // Increment the in-memory continuation id
    let k_updated = pos.ins().iadd_imm(k, 1);
    pos.ins().store(ir::MemFlags::trusted(), k_updated, k_addr, 0);


    // alloc the stack, and setup %rsp
    // TODO: alloc stack, so we have someething other than i64.const 0 here
    let newSP = pos.ins().iconst(I64, 321);

    // let stack_loc = func.locations[newSP];
    // println!("newSP loc = {:?}", stack_loc);

    pos.ins().copy_special(RU::rsp, RU::rsp);

    // args for the function call
    let mut new_args = ir::ValueList::default();
    new_args.push(callee, &mut func.dfg.value_lists);
    new_args.push(k, &mut func.dfg.value_lists);
    for i in 1..old_args.len(&func.dfg.value_lists) {
        new_args.push(
            old_args.as_slice(&func.dfg.value_lists)[i],
            &mut func.dfg.value_lists,
        );
    }


    func.dfg.replace(inst).CallIndirect(ir::Opcode::CallIndirect, ptr_ty, sig, new_args);
    println!("x86 expand control!");
    // unimplemented!()
}

pub fn expand_restore_x86(
    _inst: ir::Inst,
    _func: &mut ir::Function,
    _cfg: &mut ControlFlowGraph,
    _isa: &dyn TargetIsa,
) { 
    println!("x86 expand restore!");
    unimplemented!()
}