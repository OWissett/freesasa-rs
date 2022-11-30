#include <iostream>
#include <stdio.h>
#include <stddef.h>
#include <vector>
#include <string>

#include "freesasa.h"

freesasa_nodearea get_area(std::string pdb_path, const freesasa_classifier* classifier);

int main(int argc, char *argv[])
{
    freesasa_structure *structure = NULL;
    freesasa_result *result = NULL;

    const freesasa_classifier *classifier = &freesasa_default_classifier;
    freesasa_nodearea area;

    // Read structure from file
    std::vector<std::string> pdb_paths;
    pdb_paths.reserve(argc - 1);

    // Read file names from command line
    for (int i = 1; i < argc; ++i)
    {
        pdb_paths.push_back(argv[i]);
    }

    for (std::vector<std::string>::const_iterator it = pdb_paths.begin(); it != pdb_paths.end(); ++it)
    {
        freesasa_nodearea area;

        area = get_area(*it, classifier);
        // Print results
        std::cout << "Structure: " << *it << std::endl;
        std::cout << "Total SASA:     " << area.total << std::endl;
        std::cout << "Non-polar SASA: " << area.apolar << std::endl;
        std::cout << "Polar SASA:     " << area.polar << std::endl;
        std::cout << std::endl;
    }

    return EXIT_SUCCESS;
}

freesasa_nodearea get_area(std::string pdb_path, const freesasa_classifier *classifier)
{
    freesasa_structure *structure = NULL;
    freesasa_result *result = NULL;
    freesasa_nodearea area;

    // Read structure from file
    FILE *pdb_file = fopen(pdb_path.c_str(), "r");

    if (!pdb_file)
    {
        throw std::runtime_error("Could not open file " + pdb_path);
    }

    structure = freesasa_structure_from_pdb(pdb_file, classifier, 0);

    if (structure == NULL)
    {
        throw std::runtime_error("Error reading structure from " + pdb_path);
    }

    // Calculate solvent accessible surface area
    result = freesasa_calc_structure(structure, NULL);

    if (!result)
    {
        throw std::runtime_error("Error calculating SASA for " + pdb_path);
    }

    area = freesasa_result_classes(structure, result);

    // Free memory
    freesasa_result_free(result);
    freesasa_structure_free(structure);

    return area;
}