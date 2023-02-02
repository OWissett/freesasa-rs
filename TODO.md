## TODO

Since the `freesasa` C library is procedural/functional(as is `C`) but
Rust is procedural/functional/object oriented, two forms could be implemented:

   1. A functional way of using the library(this will still need objects)
   2. An object-oriented way of using the library

Using objects is more idiomatic to Rust, however, there will be some cases
where using a more functional approach will be more appropriate.

Bindgen creates raw FFI bindings for the C-API, however, these are not
nice to use in a rusty way since you are dealing with lots of raw pointers
and `unsafe` code blocks, which is not good. Therefore, raw FFI functions
will be wrapped in nice rusty wrappers. A typical example of this would be
the `freesasa_structure_new`, which we can place in the `Structure::new()`
function, and then store the underlying pointer, which will then be used by
`freesasa_structure_free` which is called in the `Drop` trait `drop()` function
upon deallocation of the `Structure` object.

**Structs to implement**
- [ ] structure             - protein structure object to operate on
   * `freesasa_calc_structure(*const freesasa_structure, *const parameters)`
   * `freesasa_structure_new`
   * `freesasa_calc_tree`
   * `freesasa_structure_free` - call this in `Drop`
- [ ] result                - result of freesasa_calc_structure
- [ ] result tree           - result of freesasa_calc_tree, can also be made from result object
- [ ] selection             - selection of a subset of result object(this maybe can be done as a result object, look in to this)
- [ ] node(maybe do this?) - generic data structure used to store data(tree)
- [ ] tree(see above)
- [ ] parameters            - store parameters for the calculations




### List of All functions in `bindings.rs`

I think there are about 60 functions publicly exposed. Some of these will be implemented as functions
others will be as methods or associate functions.

**Calculation Functions**
* `freesasa_calc_structure(structure: *const freesasa_structure, parameters: *const freesasa_parameters)`
* `freesasa_calc_coord(xyz: *const f64, radii: *const f64, n: std::os::raw::c_int, parameters: *const freesasa_parameters)`
* `freesasa_calc_tree(structure: *const freesasa_structure, parameters: *const freesasa_parameters, name: *const std::os::raw::c_char)`

**Structure Functions**

*constructor/destructor*
* `freesasa_structure_new()`
* `freesasa_structure_free(structure: *mut freesasa_structure)`
* `freesasa_structure_from_pdb(pdb: *mut FILE, classifier: *const freesasa_classifier, options: std::os::raw::c_int)`
* `freesasa_structure_array(pdb: *mut FILE, n: *mut std::os::raw::c_int, classifier: *const freesasa_classifier, options: std::os::raw::c_int)`

*builder*
* `freesasa_structure_add_atom(structure: *mut freesasa_structure, atom_name: *const std::os::raw::c_char, residue_name: *const std::os::raw::c_char, residue_number: *const std::os::raw::c_char, chain_label: std::os::raw::c_char, x: f64, y: f64, z: f64)`
* `freesasa_structure_add_atom_wopt(structure: *mut freesasa_structure, atom_name: *const std::os::raw::c_char, residue_name: *const std::os::raw::c_char, residue_number: *const std::os::raw::c_char, chain_label: std::os::raw::c_char, x: f64, y: f64, z: f64, classifier: *const freesasa_classifier, options: std::os::raw::c_int)`
* `freesasa_structure_add_cif_atom(structure: *mut freesasa_structure, atom: *mut freesasa_cif_atom, classifier: *const freesasa_classifier, options: std::os::raw::c_int)`

*getters*
* `freesasa_structure_get_chains(structure: *const freesasa_structure, chains: *const std::os::raw::c_char, classifier: *const freesasa_classifier, options: std::os::raw::c_int)`
* `freesasa_structure_chain_labels(structure: *const freesasa_structure)`
* `freesasa_structure_n(structure: *const freesasa_structure)`
* `freesasa_structure_n_residues(structure: *const freesasa_structure)`
* `freesasa_structure_n_chains(structure: *const freesasa_structure)`
* `freesasa_structure_residue_atoms(structure: *const freesasa_structure, r_i: std::os::raw::c_int, first: *mut std::os::raw::c_int, last: *mut std::os::raw::c_int)`
* `freesasa_structure_chain_atoms(structure: *const freesasa_structure, chain: std::os::raw::c_char, first: *mut std::os::raw::c_int, last: *mut std::os::raw::c_int)`
* `freesasa_structure_chain_residues(structure: *const freesasa_structure, chain: std::os::raw::c_char, first: *mut std::os::raw::c_int, last: *mut std::os::raw::c_int)`
* `freesasa_structure_classifier_name(structure: *const freesasa_structure)`
* `freesasa_structure_coord_array(structure: *const freesasa_structure)`

*structure radius*
* `freesasa_structure_radius(structure: *const freesasa_structure)`
* `freesasa_structure_set_radius(structure: *mut freesasa_structure, radii: *const f64)`

*atom*
* `freesasa_structure_atom_name(structure: *const freesasa_structure, i: std::os::raw::c_int)`
* `freesasa_structure_atom_res_name(structure: *const freesasa_structure, i: std::os::raw::c_int)`
* `freesasa_structure_atom_res_number(structure: *const freesasa_structure, i: std::os::raw::c_int)`
* `freesasa_structure_atom_chain(structure: *const freesasa_structure, i: std::os::raw::c_int)`
* `freesasa_structure_atom_symbol(structure: *const freesasa_structure, i: std::os::raw::c_int)`
* `freesasa_structure_atom_radius(structure: *const freesasa_structure, i: std::os::raw::c_int)`
* `freesasa_structure_atom_set_radius(structure: *mut freesasa_structure, i: std::os::raw::c_int, radius: f64)`

*residue*
* `freesasa_structure_residue_name(structure: *const freesasa_structure, r_i: std::os::raw::c_int)`
* `freesasa_structure_residue_number(structure: *const freesasa_structure, r_i: std::os::raw::c_int)`
* `freesasa_structure_residue_chain(structure: *const freesasa_structure, r_i: std::os::raw::c_int)`

*model*
* `freesasa_structure_model(structure: *const freesasa_structure)`
* `freesasa_structure_set_model(structure: *mut freesasa_structure, model: std::os::raw::c_int)`

**Result Functions**
* `freesasa_result_classes(structure: *const freesasa_structure, result: *const freesasa_result)`
* `freesasa_result_free(result: *mut freesasa_result)`

**Classifier Functions**

*constructor/destructor*
* `freesasa_classifier_from_file(file: *mut FILE)`
* `freesasa_classifier_free(classifier: *mut freesasa_classifier)`

*other*
* `freesasa_classifier_radius(classifier: *const freesasa_classifier, res_name: *const std::os::raw::c_char, atom_name: *const std::os::raw::c_char)`
* `freesasa_classifier_class(classifier: *const freesasa_classifier, res_name: *const std::os::raw::c_char, atom_name: *const std::os::raw::c_char)`
* `freesasa_classifier_class2str(atom_class: freesasa_atom_class)`
* `freesasa_classifier_name(classifier: *const freesasa_classifier)`

**Selection Functions**

*constructor/destructors*
* `freesasa_selection_new(command: *const std::os::raw::c_char, structure: *const freesasa_structure, result: *const freesasa_result)`
* `freesasa_selection_free(selection: *mut freesasa_selection)`

*getters*
* `freesasa_selection_name(selection: *const freesasa_selection)`
* `freesasa_selection_command(selection: *const freesasa_selection)`
* `freesasa_selection_area(selection: *const freesasa_selection)`
* `freesasa_selection_n_atoms(selection: *const freesasa_selection)`

*other operations*
* `freesasa_select_area(command: *const std::os::raw::c_char, name: *mut std::os::raw::c_char, area: *mut f64, structure: *const freesasa_structure, result: *const freesasa_result)`


**Logging Functions**
* `freesasa_set_verbosity(v: freesasa_verbosity)`
* `freesasa_get_verbosity()`
* `freesasa_set_err_out(err: *mut FILE)`
* `freesasa_get_err_out()`

**Tree Functions**
* `freesasa_tree_new()`
* `freesasa_tree_init(result: *const freesasa_result, structure: *const freesasa_structure, name: *const std::os::raw::c_char)`
* `freesasa_tree_add_result(tree: *mut freesasa_node, result: *const freesasa_result, structure: *const freesasa_structure, name: *const std::os::raw::c_char)`
* `freesasa_tree_join(tree1 : *mut freesasa_node, tree2 : *mut *mut freesasa_node)`
* `freesasa_tree_export(output: *mut FILE, root: *mut freesasa_node, options: std::os::raw::c_int)`

**Node Functions**

*generic*
* `freesasa_node_free(root: *mut freesasa_node)`
* `freesasa_node_area(node: *const freesasa_node)`
* `freesasa_node_children(node: *mut freesasa_node)`
* `freesasa_node_next(node: *mut freesasa_node)`
* `freesasa_node_parent(node: *mut freesasa_node)`
* `freesasa_node_type(node: *const freesasa_node)`
* `freesasa_node_name(node: *const freesasa_node)`

*classifier*
* `freesasa_node_classified_by(node: *const freesasa_node)`

*atom*
* `freesasa_node_atom_is_polar(node: *const freesasa_node)`
* `freesasa_node_atom_is_mainchain(node: *const freesasa_node)`
* `freesasa_node_atom_radius(node: *const freesasa_node)`
* `freesasa_node_atom_pdb_line(node: *const freesasa_node)`
* `freesasa_node_atom_residue_number(node: *const freesasa_node)`
* `freesasa_node_atom_residue_name(node: *const freesasa_node)`
* `freesasa_node_atom_chain(node: *const freesasa_node)`

*residue*
* `freesasa_node_residue_number(node: *const freesasa_node)`
* `freesasa_node_residue_n_atoms(node: *const freesasa_node)`
* `freesasa_node_residue_reference(node: *const freesasa_node)`

*chain*
* `freesasa_node_chain_n_residues(node: *const freesasa_node)`

*structure*
* `freesasa_node_structure_n_chains(node: *const freesasa_node)`
* `freesasa_node_structure_n_atoms(node: *const freesasa_node)`
* `freesasa_node_structure_chain_labels(node: *const freesasa_node)`
* `freesasa_node_structure_model(node: *const freesasa_node)`
* `freesasa_node_structure_result(node: *const freesasa_node)`
* `freesasa_node_structure_selections(node: *const freesasa_node)`
* `freesasa_node_structure_add_selection(node: *mut freesasa_node, selection: *const freesasa_selection)`

*parameters*
* `freesasa_node_result_parameters(node: *const freesasa_node)`
